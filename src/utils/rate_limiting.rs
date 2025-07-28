use anyhow::Result;
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{warn, debug, error};
use thiserror::Error;

use crate::utils::constants::*;
use crate::utils::helpers::is_public_rpc_endpoint;

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for provider {provider}: {current}/{limit} requests")]
    Exceeded { provider: String, current: u32, limit: u32 },
    
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Timeout waiting for rate limit: {0:?}")]
    Timeout(Duration),
}

/// CRÍTICO 2024: Rate limiter para RPC providers profissionais
#[derive(Clone)]
pub struct RpcRateLimiter {
    limiters: Arc<RwLock<HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState>>>>>,
    provider_configs: Arc<RwLock<HashMap<String, ProviderConfig>>>,
    request_counters: Arc<RwLock<HashMap<String, RequestCounter>>>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub rate_limit_rps: u32,
    pub max_concurrent: u32,
    pub burst_size: u32,
    pub timeout: Duration,
    pub priority: u8, // 1-10, higher = better
}

#[derive(Debug, Clone)]
struct RequestCounter {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    last_request_time: Instant,
    current_concurrent: u32,
}

impl Default for RequestCounter {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_request_time: Instant::now(),
            current_concurrent: 0,
        }
    }
}

impl RpcRateLimiter {
    /// Criar novo rate limiter com configurações para providers profissionais
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            provider_configs: Arc::new(RwLock::new(HashMap::new())),
            request_counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// CRÍTICO 2024: Configurar providers profissionais com rate limits apropriados
    pub async fn configure_providers(&self) -> Result<()> {
        let mut configs = self.provider_configs.write().await;
        let mut limiters = self.limiters.write().await;
        let mut counters = self.request_counters.write().await;
        
        // Helius (Recomendado para Solana)
        let helius_config = ProviderConfig {
            name: "helius".to_string(),
            rate_limit_rps: 500, // 500 RPS no plano pago
            max_concurrent: 50,
            burst_size: 100,
            timeout: Duration::from_secs(30),
            priority: 10, // Prioridade máxima
        };
        
        // QuickNode
        let quicknode_config = ProviderConfig {
            name: "quicknode".to_string(),
            rate_limit_rps: 500, // 500 RPS no plano pago
            max_concurrent: 50,
            burst_size: 100,
            timeout: Duration::from_secs(30),
            priority: 9,
        };
        
        // Alchemy
        let alchemy_config = ProviderConfig {
            name: "alchemy".to_string(),
            rate_limit_rps: 120, // 120 RPS limite conhecido
            max_concurrent: 20,
            burst_size: 50,
            timeout: Duration::from_secs(30),
            priority: 8,
        };
        
        // Endpoints públicos - INADEQUADOS para produção
        let public_config = ProviderConfig {
            name: "public".to_string(),
            rate_limit_rps: 10, // 100 req/10s = 10 RPS
            max_concurrent: 5,
            burst_size: 10,
            timeout: Duration::from_secs(10),
            priority: 1, // Prioridade mínima
        };
        
        // Configurar rate limiters
        for config in [&helius_config, &quicknode_config, &alchemy_config, &public_config] {
            let quota = Quota::per_second(std::num::NonZeroU32::new(config.rate_limit_rps).unwrap());
            let limiter = Arc::new(RateLimiter::direct(quota));
            
            limiters.insert(config.name.clone(), limiter);
            configs.insert(config.name.clone(), config.clone());
            counters.insert(config.name.clone(), RequestCounter::default());
        }
        
        debug!("Configured rate limiters for {} providers", configs.len());
        Ok(())
    }
    
    /// Detectar provider baseado na URL
    pub fn detect_provider(&self, url: &str) -> String {
        if url.contains("helius-rpc.com") {
            "helius".to_string()
        } else if url.contains("quiknode.pro") {
            "quicknode".to_string()
        } else if url.contains("g.alchemy.com") {
            "alchemy".to_string()
        } else if is_public_rpc_endpoint(url) {
            warn!("AVISO: Usando endpoint público inadequado para produção: {}", url);
            "public".to_string()
        } else {
            warn!("Provider desconhecido: {}, usando configuração padrão", url);
            "public".to_string()
        }
    }
    
    /// CRÍTICO: Aguardar permissão para fazer request (rate limiting)
    pub async fn wait_for_permit(&self, provider: &str) -> Result<RateLimitPermit, RateLimitError> {
        let limiters = self.limiters.read().await;
        let configs = self.provider_configs.read().await;
        
        let limiter = limiters.get(provider)
            .ok_or_else(|| RateLimitError::ProviderNotFound(provider.to_string()))?;
        
        let config = configs.get(provider)
            .ok_or_else(|| RateLimitError::ProviderNotFound(provider.to_string()))?;
        
        // Verificar limite de concurrent requests
        {
            let mut counters = self.request_counters.write().await;
            let counter = counters.get_mut(provider)
                .ok_or_else(|| RateLimitError::ProviderNotFound(provider.to_string()))?;
            
            if counter.current_concurrent >= config.max_concurrent {
                return Err(RateLimitError::Exceeded {
                    provider: provider.to_string(),
                    current: counter.current_concurrent,
                    limit: config.max_concurrent,
                });
            }
            
            counter.current_concurrent += 1;
            counter.total_requests += 1;
            counter.last_request_time = Instant::now();
        }
        
        // Aguardar rate limit permit
        match tokio::time::timeout(config.timeout, limiter.until_ready()).await {
            Ok(_) => {
                debug!("Rate limit permit acquired for provider: {}", provider);
                Ok(RateLimitPermit {
                    provider: provider.to_string(),
                    rate_limiter: Arc::clone(&self.request_counters),
                })
            }
            Err(_) => {
                // Decrementar contador se timeout
                let mut counters = self.request_counters.write().await;
                if let Some(counter) = counters.get_mut(provider) {
                    counter.current_concurrent = counter.current_concurrent.saturating_sub(1);
                    counter.failed_requests += 1;
                }
                Err(RateLimitError::Timeout(config.timeout))
            }
        }
    }
    
    /// Obter estatísticas de rate limiting
    pub async fn get_stats(&self) -> HashMap<String, ProviderStats> {
        let counters = self.request_counters.read().await;
        let configs = self.provider_configs.read().await;
        
        let mut stats = HashMap::new();
        
        for (provider, counter) in counters.iter() {
            if let Some(config) = configs.get(provider) {
                let success_rate = if counter.total_requests > 0 {
                    (counter.successful_requests as f64 / counter.total_requests as f64) * 100.0
                } else {
                    0.0
                };
                
                stats.insert(provider.clone(), ProviderStats {
                    provider: provider.clone(),
                    total_requests: counter.total_requests,
                    successful_requests: counter.successful_requests,
                    failed_requests: counter.failed_requests,
                    current_concurrent: counter.current_concurrent,
                    success_rate,
                    rate_limit_rps: config.rate_limit_rps,
                    max_concurrent: config.max_concurrent,
                    priority: config.priority,
                });
            }
        }
        
        stats
    }
    
    /// NOVO 2024: Obter melhor provider disponível baseado em performance
    pub async fn get_best_provider(&self) -> Option<String> {
        let stats = self.get_stats().await;
        
        // Filtrar providers com boa performance
        let mut candidates: Vec<_> = stats.values()
            .filter(|stat| {
                stat.success_rate >= 95.0 && // 95% success rate mínimo
                stat.current_concurrent < (stat.max_concurrent * 80 / 100) // < 80% capacity
            })
            .collect();
        
        // Ordenar por prioridade e depois por success rate
        candidates.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(b.success_rate.partial_cmp(&a.success_rate).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        candidates.first().map(|stat| stat.provider.clone())
    }
}

/// Permit que deve ser mantido durante a request
pub struct RateLimitPermit {
    provider: String,
    rate_limiter: Arc<RwLock<HashMap<String, RequestCounter>>>,
}

impl Drop for RateLimitPermit {
    fn drop(&mut self) {
        let provider = self.provider.clone();
        let rate_limiter = Arc::clone(&self.rate_limiter);
        
        // Spawn task para decrementar contador
        tokio::spawn(async move {
            let mut counters = rate_limiter.write().await;
            if let Some(counter) = counters.get_mut(&provider) {
                counter.current_concurrent = counter.current_concurrent.saturating_sub(1);
            }
        });
    }
}

impl RateLimitPermit {
    /// Marcar request como bem-sucedida
    pub async fn mark_success(&self) {
        let mut counters = self.rate_limiter.write().await;
        if let Some(counter) = counters.get_mut(&self.provider) {
            counter.successful_requests += 1;
        }
    }
    
    /// Marcar request como falha
    pub async fn mark_failure(&self) {
        let mut counters = self.rate_limiter.write().await;
        if let Some(counter) = counters.get_mut(&self.provider) {
            counter.failed_requests += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderStats {
    pub provider: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub current_concurrent: u32,
    pub success_rate: f64,
    pub rate_limit_rps: u32,
    pub max_concurrent: u32,
    pub priority: u8,
}

/// NOVO 2024: Helper para calcular delay baseado em rate limit
pub fn calculate_adaptive_delay(
    current_rate: u32,
    target_rate: u32,
    error_rate: f64,
) -> Duration {
    if current_rate > target_rate {
        // Se excedendo rate limit, aumentar delay
        let excess_ratio = current_rate as f64 / target_rate as f64;
        Duration::from_millis((1000.0 * excess_ratio) as u64)
    } else if error_rate > 0.1 {
        // Se muitos erros, diminuir rate
        Duration::from_millis(1500)
    } else {
        // Rate normal
        Duration::from_millis(1000 / target_rate as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_rate_limiter_creation() {
        let limiter = RpcRateLimiter::new();
        limiter.configure_providers().await.unwrap();
        
        let stats = limiter.get_stats().await;
        assert!(!stats.is_empty());
        assert!(stats.contains_key("helius"));
        assert!(stats.contains_key("quicknode"));
    }
    
    #[test]
    async fn test_provider_detection() {
        let limiter = RpcRateLimiter::new();
        
        assert_eq!(limiter.detect_provider("https://mainnet.helius-rpc.com"), "helius");
        assert_eq!(limiter.detect_provider("https://api.mainnet-beta.solana.com"), "public");
    }
    
    #[test]
    async fn test_rate_limit_permit() {
        let limiter = RpcRateLimiter::new();
        limiter.configure_providers().await.unwrap();
        
        let permit = limiter.wait_for_permit("helius").await.unwrap();
        permit.mark_success().await;
        
        let stats = limiter.get_stats().await;
        let helius_stats = stats.get("helius").unwrap();
        assert_eq!(helius_stats.successful_requests, 1);
    }
} 