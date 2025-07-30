use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};
use solana_client::rpc_client::RpcClient;

mod config;
mod dex;
mod monitoring;
mod strategy;
mod execution;
mod math;
mod risk;
mod utils;
mod graph;

use config::BotConfig;
use monitoring::PriceMonitor;
use strategy::StrategyEngine;
use execution::ExecutionEngine;
use risk::RiskManager;
// NOVO 2024: Imports obrigatórios para funcionalidades críticas
use utils::rate_limiting::{RpcRateLimiter, RateLimitError};
use utils::priority_fees::DynamicPriorityFeeCalculator;
use utils::constants::*;
use utils::helpers::{is_public_rpc_endpoint, is_valid_rpc_provider};

pub struct ArbitrageBot {
    pub config: Arc<BotConfig>,
    pub price_monitor: Arc<PriceMonitor>,
    pub strategy_engine: Arc<StrategyEngine>,
    pub execution_engine: Arc<ExecutionEngine>,
    pub risk_manager: Arc<RiskManager>,
    // NOVO 2024: Componentes críticos obrigatórios
    pub rpc_rate_limiter: Arc<RpcRateLimiter>,
    pub priority_fee_calculator: Arc<DynamicPriorityFeeCalculator>,
    pub rpc_client: Arc<RpcClient>,
}

impl ArbitrageBot {
    pub async fn new(config: BotConfig) -> Result<Self> {
        let config = Arc::new(config);
        
        // CRÍTICO 2024: Validar configuração antes de continuar
        Self::validate_critical_config(&config).await?;
        
        // CRÍTICO 2024: Configurar RPC client com provider profissional
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            config.rpc.primary_url.clone(),
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        ));
        
        // NOVO 2024: Inicializar rate limiter obrigatório
        let rpc_rate_limiter = Arc::new(RpcRateLimiter::new());
        rpc_rate_limiter.configure_providers().await?;
        
        // NOVO 2024: Inicializar calculadora de priority fees dinâmicas
        let priority_fee_calculator = Arc::new(DynamicPriorityFeeCalculator::new(
            Arc::clone(&rpc_client),
            crate::execution::jito::MevProtectionConfig {
                max_priority_fee_lamports: config.trading.max_priority_fee_lamports,
                min_priority_fee_lamports: config.trading.base_priority_fee,
                dynamic_adjustment: config.mev_protection.priority_fee_dynamic,
                slippage_adaptive: config.mev_protection.slippage_adaptive,
            },
        ));
        
        // Inicializar componentes principais
        let price_monitor = Arc::new(PriceMonitor::new(config.clone()).await?);
        let strategy_engine = Arc::new(StrategyEngine::new(config.clone()));
        let execution_engine = Arc::new(ExecutionEngine::new(config.clone()).await?);
        let risk_manager = Arc::new(RiskManager::new(config.clone()));
        
        info!("Bot de arbitragem BONK/SOL inicializado com configurações 2024");
        info!("RPC Provider: {}", Self::detect_provider_type(&config.rpc.primary_url));
        info!("MEV Protection: Priority fees dinâmicas habilitadas");
        info!("Rate Limiting: Configurado para provider profissional");
        
        Ok(Self {
            config,
            price_monitor,
            strategy_engine,
            execution_engine,
            risk_manager,
            rpc_rate_limiter,
            priority_fee_calculator,
            rpc_client,
        })
    }
    
    /// CRÍTICO 2024: Validar configurações críticas obrigatórias
    async fn validate_critical_config(config: &BotConfig) -> Result<()> {
        // Validar RPC provider profissional
        if is_public_rpc_endpoint(&config.rpc.primary_url) {
            return Err(anyhow::anyhow!(
                "ERRO CRÍTICO: Endpoint público detectado em produção: {}. Use Helius, QuickNode ou Alchemy.",
                config.rpc.primary_url
            ));
        }
        
        if !is_valid_rpc_provider(&config.rpc.primary_url) {
            warn!(
                "AVISO: Provider RPC desconhecido: {}. Certifique-se de que é um provider profissional.",
                config.rpc.primary_url
            );
        }
        
        // Validar MEV protection atualizada
        if config.mev_protection.jito_enabled {
            warn!("AVISO: Jito mempool público foi suspenso em março 2024. Use apenas bundles privados.");
        }
        
        if !config.mev_protection.priority_fee_dynamic {
            return Err(anyhow::anyhow!(
                "ERRO: Priority fees dinâmicas são obrigatórias em 2024"
            ));
        }
        
        // Validar rate limiting
        if config.rate_limiting.requests_per_second < 50 {
            warn!(
                "AVISO: Rate limit muito baixo ({} RPS) para arbitragem eficiente",
                config.rate_limiting.requests_per_second
            );
        }
        
        Ok(())
    }
    
    /// Detectar tipo de provider para logging
    fn detect_provider_type(url: &str) -> &'static str {
        if url.contains("helius-rpc.com") {
            "Helius (Recomendado)"
        } else if url.contains("quiknode.pro") {
            "QuickNode"
        } else if url.contains("g.alchemy.com") {
            "Alchemy"
        } else if url.contains("chainstack.com") {
            "Chainstack"
        } else if is_public_rpc_endpoint(url) {
            "Público (INADEQUADO)"
        } else {
            "Desconhecido"
        }
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("Iniciando Bot de Arbitragem BONK/SOL Solana");
        
        // NOVO 2024: Verificar saúde do RPC provider antes de iniciar
        self.verify_rpc_health().await?;
        
        // Start price monitoring
        let monitor_handle = {
            let price_monitor = self.price_monitor.clone();
            tokio::spawn(async move {
                if let Err(e) = price_monitor.start().await {
                    error!("Price monitor falhou: {}", e);
                }
            })
        };
        
        // NOVO 2024: Start metrics server com Prometheus
        let metrics_handle = {
            let config = self.config.clone();
            tokio::spawn(async move {
                if let Err(e) = start_metrics_server(config.monitoring.metrics_port).await {
                    error!("Metrics server falhou: {}", e);
                }
            })
        };
        
        // NOVO 2024: Start rate limiter stats monitoring
        let rate_limiter_stats_handle = {
            let rate_limiter = self.rpc_rate_limiter.clone();
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let stats = rate_limiter.get_stats().await;
                    for (provider, stat) in stats {
                        info!(
                            "RPC Stats [{}]: {}% success, {}/{} concurrent, {} total requests",
                            provider,
                            stat.success_rate,
                            stat.current_concurrent,
                            stat.max_concurrent,
                            stat.total_requests
                        );
                    }
                }
            })
        };
        
        // Main arbitrage loop - ATUALIZADO com rate limiting
        let mut scan_interval = interval(Duration::from_millis(self.config.performance.scan_interval_ms));
        
        loop {
            scan_interval.tick().await;
            
            // NOVO 2024: Rate limiting obrigatório antes de cada ciclo
            let provider = self.rpc_rate_limiter.detect_provider(&self.config.rpc.primary_url);
            
            match self.rpc_rate_limiter.wait_for_permit(&provider).await {
                Ok(permit) => {
                    // Executar ciclo de arbitragem
                    match self.execute_arbitrage_cycle().await {
                        Ok(()) => {
                            permit.mark_success().await;
                        }
                        Err(e) => {
                            error!("Ciclo de arbitragem falhou: {}", e);
                            permit.mark_failure().await;
                        }
                    }
                }
                Err(RateLimitError::Exceeded { provider, current, limit }) => {
                    warn!(
                        "Rate limit excedido para {}: {}/{} requests. Aguardando...",
                        provider, current, limit
                    );
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => {
                    error!("Erro de rate limiting: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }
    
    /// NOVO 2024: Executar ciclo de arbitragem com priority fees dinâmicas
    async fn execute_arbitrage_cycle(&self) -> Result<()> {
        // Get current prices
        let prices = self.price_monitor.get_current_prices().await;
        
        // Find arbitrage opportunities
        let opportunities = self.strategy_engine.find_opportunities(&prices).await;
        
        for opportunity in opportunities {
            // Risk check
            if !self.risk_manager.validate_opportunity(&opportunity).await {
                continue;
            }
            
            // NOVO 2024: Obter priority fee dinâmico para esta oportunidade
            let priority_fee_recommendation = self.priority_fee_calculator
                .get_recommended_fee()
                .await?;
            
            info!(
                "Executando arbitragem com priority fee: {} lamports (congestionamento: {:?})",
                priority_fee_recommendation.recommended_fee,
                priority_fee_recommendation.congestion_level
            );
            
            // Execute arbitrage com priority fee dinâmico
            match self.execution_engine.execute_arbitrage_with_priority_fee(
                opportunity,
                priority_fee_recommendation,
            ).await {
                Ok(signature) => {
                    info!("Arbitragem executada com sucesso: {}", signature);
                    crate::utils::metrics::ARBITRAGE_SUCCESSES.inc();
                }
                Err(e) => {
                    error!("Falha ao executar arbitragem: {}", e);
                }
            }
            
            crate::utils::metrics::ARBITRAGE_ATTEMPTS.inc();
        }
        
        Ok(())
    }
    
    /// NOVO 2024: Verificar saúde do RPC provider
    async fn verify_rpc_health(&self) -> Result<()> {
        info!("Verificando saúde do RPC provider...");
        
        let start_time = std::time::Instant::now();
        
        // Test basic RPC call
        match self.rpc_client.get_health().await {
            Ok(_) => {
                let latency = start_time.elapsed();
                info!("RPC provider responsivo (latência: {:?})", latency);
                
                if latency > Duration::from_millis(1000) {
                    warn!("Alta latência RPC detectada: {:?}", latency);
                }
                
                Ok(())
            }
            Err(e) => {
                error!("RPC provider não responsivo: {}", e);
                Err(anyhow::anyhow!("RPC health check falhou: {}", e))
            }
        }
    }
    
    /// NOVO 2024: Obter estatísticas do bot
    pub async fn get_bot_stats(&self) -> BotStats {
        let rate_limiter_stats = self.rpc_rate_limiter.get_stats().await;
        let congestion_stats = self.priority_fee_calculator.get_congestion_stats().await;
        
        BotStats {
            rpc_provider_stats: rate_limiter_stats,
            congestion_stats,
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default(),
        }
    }
}

/// NOVO 2024: Estrutura de estatísticas do bot
#[derive(Debug, Clone)]
pub struct BotStats {
    pub rpc_provider_stats: std::collections::HashMap<String, utils::rate_limiting::ProviderStats>,
    pub congestion_stats: utils::priority_fees::CongestionStats,
    pub uptime: Duration,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging - ATUALIZADO 2024
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json() // JSON format para produção
        .init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    info!("=== Solana Arbitrage Bot 2024 ===");
    info!("Versão: 2.0 (Atualizações críticas implementadas)");
    info!("Features: Priority fees dinâmicas, Rate limiting, RPC profissional");
    
    // Load configuration - com validação crítica 2024
    let config = match BotConfig::from_env() {
        Ok(config) => {
            info!("Configuração carregada com sucesso");
            config
        }
        Err(e) => {
            error!("Falha ao carregar configuração: {}", e);
            error!("Verifique se as variáveis de ambiente ARB_BOT_* estão configuradas");
            error!("Use um RPC provider profissional (Helius/QuickNode/Alchemy)");
            return Err(e.into());
        }
    };
    
    // Initialize and start bot
    let mut bot = match ArbitrageBot::new(config).await {
        Ok(bot) => {
            info!("Bot inicializado com sucesso");
            bot
        }
        Err(e) => {
            error!("Falha ao inicializar bot: {}", e);
            return Err(e);
        }
    };
    
    // Start bot with graceful shutdown
    match bot.start().await {
        Ok(()) => {
            info!("Bot finalizado graciosamente");
            Ok(())
        }
        Err(e) => {
            error!("Bot falhou: {}", e);
            Err(e)
        }
    }
}

/// NOVO 2024: Servidor de métricas Prometheus funcional
async fn start_metrics_server(port: u16) -> Result<()> {
    use axum::{routing::get, Router};
    use std::net::SocketAddr;
    
    info!("Iniciando servidor de métricas na porta {}", port);
    
    // NOVO 2024: Configurar rotas de métricas
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    info!("Métricas disponíveis em http://0.0.0.0:{}/metrics", port);
    info!("Health check disponível em http://0.0.0.0:{}/health", port);
    
    // Use the newer axum API
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Servidor de métricas falhou: {}", e))
}

/// Handler de métricas Prometheus
async fn metrics_handler() -> String {
    // TODO: Implementar coleta real de métricas
    format!(
        "# HELP arbitrage_attempts_total Total de tentativas de arbitragem\n\
         # TYPE arbitrage_attempts_total counter\n\
         arbitrage_attempts_total {}\n\
         # HELP arbitrage_successes_total Total de arbitragens bem-sucedidas\n\
         # TYPE arbitrage_successes_total counter\n\
         arbitrage_successes_total {}\n",
        0, 0 // Placeholder - implementar métricas reais
    )
}

/// Handler de health check
async fn health_handler() -> &'static str {
    "OK"
}

/// Handler de estatísticas
async fn stats_handler() -> String {
    serde_json::json!({
        "status": "running",
        "version": "2.0",
        "features": [
            "priority_fees_dynamic",
            "rate_limiting",
            "rpc_professional",
            "mev_protection_updated"
        ],
        "timestamp": chrono::Utc::now().to_rfc3339()
    }).to_string()
} 