use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotConfig {
    pub rpc: RpcConfig,
    pub wallet: WalletConfig,
    pub trading: TradingConfig,
    pub dex: DexConfig,
    pub risk: RiskConfig,
    pub monitoring: MonitoringConfig,
    pub performance: PerformanceConfig,
    // NOVO 2024: Configurações críticas atualizadas
    pub mev_protection: MevProtectionConfig,
    pub rate_limiting: RateLimitingConfig,
    pub failover: FailoverConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcConfig {
    /// OBRIGATÓRIO 2024: Provider profissional (Helius/QuickNode/Alchemy)
    pub primary_url: String,
    /// OBRIGATÓRIO: URLs de backup para failover
    pub backup_urls: Vec<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    /// DESATUALIZADO: Jito engine (mempool público suspenso março 2024)
    pub jito_engine: Option<String>,
    pub yellowstone_endpoint: Option<String>,
    /// NOVO 2024: Rate limit baseado no plano do provider
    pub rate_limit_rps: u32,
    /// NOVO 2024: Máximo de conexões concorrentes
    pub max_concurrent_requests: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletConfig {
    pub private_key: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfig {
    pub max_slippage_bps: u16,
    pub max_accounts: u16,
    pub min_profit_threshold: f64,
    pub max_position_size: u64,
    /// ATUALIZADO 2024: Priority fee base para cálculo dinâmico
    pub base_priority_fee: u64,
    /// NOVO 2024: Priority fee máximo permitido
    pub max_priority_fee_lamports: u64,
    /// NOVO 2024: Slippage adaptativo baseado em volatilidade
    pub slippage_adaptive: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexConfig {
    pub exclude: Vec<String>,
    pub priorities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    pub max_daily_loss: f64,
    pub max_consecutive_failures: u32,
    pub circuit_breaker_enabled: bool,
    /// NOVO 2024: Perda máxima em BPS
    pub max_loss_bps: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub log_level: String,
    pub prometheus_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    pub scan_interval_ms: u64,
    pub compute_unit_limit: u32,
    pub max_retries: u32,
}

/// NOVO 2024: Configuração de MEV Protection atualizada
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MevProtectionConfig {
    /// CRÍTICO: Jito mempool público suspenso março 2024
    pub jito_enabled: bool, // Default: false
    /// Tip mínimo obrigatório para bundles privados
    pub jito_min_tip_lamports: u64, // Default: 10000
    /// OBRIGATÓRIO: Priority fees dinâmicas
    pub priority_fee_dynamic: bool, // Default: true
    /// Slippage adaptativo durante alta volatilidade
    pub slippage_adaptive: bool, // Default: true
    /// Transaction splitting para trades grandes
    pub transaction_splitting_enabled: bool,
    /// Timing optimization para evitar picos MEV
    pub timing_optimization: bool,
}

/// NOVO 2024: Rate limiting baseado no provider
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitingConfig {
    /// Requests por segundo baseado no plano
    pub requests_per_second: u32,
    /// Máximo de requests concorrentes
    pub max_concurrent_requests: u32,
    /// Batch size para operações em lote
    pub batch_size: u32,
    /// Timeout para burst requests
    pub burst_timeout_ms: u64,
}

/// NOVO 2024: Configuração de failover RPC
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FailoverConfig {
    /// Intervalo de health checks em segundos
    pub health_check_interval_seconds: u64,
    /// Threshold de latência para failover (ms)
    pub failover_threshold_ms: u64,
    /// Rotação automática de RPC
    pub rpc_rotation_enabled: bool,
    /// Máximo de tentativas antes de failover
    pub max_failures_before_failover: u32,
}

/// NOVO 2024: Configurações de segurança hardening
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// Validação rigorosa de inputs
    pub strict_input_validation: bool,
    /// Matemática segura obrigatória
    pub safe_math_operations: bool,
    /// Whitelist de tokens permitidos
    pub token_whitelist: Vec<String>,
    /// API keys não expostas em logs
    pub secure_logging: bool,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid configuration value for field '{field}': {message}")]
    InvalidValue { field: String, message: String },
    
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
    
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] envy::Error),
    
    /// NOVO 2024: Erro de provider RPC inválido
    #[error("Invalid RPC provider configuration: {0}")]
    InvalidRpcProvider(String),
    
    /// NOVO 2024: Erro de configuração MEV
    #[error("Invalid MEV protection configuration: {0}")]
    InvalidMevConfig(String),
}

impl BotConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = envy::from_env::<Self>()
            .context("Failed to load configuration from environment")?;
        
        config.validate()
            .context("Configuration validation failed")?;
        
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        // CRÍTICO 2024: Validar RPC provider profissional
        self.validate_rpc_provider()?;
        
        // CRÍTICO 2024: Validar configuração MEV atualizada
        self.validate_mev_config()?;
        
        // Validar RPC configuration
        if self.rpc.timeout_seconds == 0 {
            return Err(ConfigError::InvalidValue {
                field: "rpc.timeout_seconds".to_string(),
                message: "Timeout must be greater than 0".to_string(),
            });
        }
        
        // NOVO 2024: Validar rate limiting
        if self.rate_limiting.requests_per_second == 0 {
            return Err(ConfigError::InvalidValue {
                field: "rate_limiting.requests_per_second".to_string(),
                message: "RPS must be greater than 0".to_string(),
            });
        }
        
        // Validar trading parameters
        if self.trading.min_profit_threshold <= 0.0 {
            return Err(ConfigError::InvalidValue {
                field: "trading.min_profit_threshold".to_string(),
                message: "Minimum profit threshold must be positive".to_string(),
            });
        }
        
        if self.trading.max_slippage_bps > 10000 {
            return Err(ConfigError::InvalidValue {
                field: "trading.max_slippage_bps".to_string(),
                message: "Maximum slippage cannot exceed 100%".to_string(),
            });
        }
        
        // NOVO 2024: Validar priority fees
        if self.trading.max_priority_fee_lamports < self.trading.base_priority_fee {
            return Err(ConfigError::InvalidValue {
                field: "trading.max_priority_fee_lamports".to_string(),
                message: "Max priority fee must be >= base priority fee".to_string(),
            });
        }
        
        // Validar wallet configuration
        if self.wallet.private_key.is_empty() {
            return Err(ConfigError::MissingRequired("wallet.private_key".to_string()));
        }
        
        if self.wallet.public_key.is_empty() {
            return Err(ConfigError::MissingRequired("wallet.public_key".to_string()));
        }
        
        // Validar risk parameters
        if self.risk.max_daily_loss <= 0.0 || self.risk.max_daily_loss > 1.0 {
            return Err(ConfigError::InvalidValue {
                field: "risk.max_daily_loss".to_string(),
                message: "Max daily loss must be between 0 and 1".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// CRÍTICO 2024: Validar que está usando provider profissional
    fn validate_rpc_provider(&self) -> Result<(), ConfigError> {
        let url = &self.rpc.primary_url;
        
        // Verificar se não está usando endpoints públicos
        if url.contains("api.mainnet-beta.solana.com") ||
           url.contains("solana-api.projectserum.com") {
            return Err(ConfigError::InvalidRpcProvider(
                "Endpoints públicos inadequados para produção. Use Helius, QuickNode ou Alchemy".to_string()
            ));
        }
        
        // Verificar providers profissionais conhecidos
        let valid_providers = [
            "helius-rpc.com",
            "quiknode.pro",
            "g.alchemy.com",
            "chainstack.com",
        ];
        
        if !valid_providers.iter().any(|provider| url.contains(provider)) {
            return Err(ConfigError::InvalidRpcProvider(
                format!("Provider desconhecido: {}. Use Helius, QuickNode, Alchemy ou Chainstack", url)
            ));
        }
        
        // Verificar rate limit mínimo
        if self.rpc.rate_limit_rps < 100 {
            return Err(ConfigError::InvalidRpcProvider(
                "Rate limit muito baixo para arbitragem. Mínimo 100 RPS recomendado".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// CRÍTICO 2024: Validar configuração MEV atualizada
    fn validate_mev_config(&self) -> Result<(), ConfigError> {
        // Jito mempool público suspenso - deve estar desabilitado por padrão
        if self.mev_protection.jito_enabled {
            eprintln!("AVISO: Jito mempool público foi suspenso em março 2024");
        }
        
        // Tip mínimo obrigatório se Jito habilitado
        if self.mev_protection.jito_enabled && 
           self.mev_protection.jito_min_tip_lamports < 10000 {
            return Err(ConfigError::InvalidMevConfig(
                "Tip mínimo deve ser >= 10.000 lamports para bundles privados".to_string()
            ));
        }
        
        // Priority fees dinâmicas obrigatórias
        if !self.mev_protection.priority_fee_dynamic {
            return Err(ConfigError::InvalidMevConfig(
                "Priority fees dinâmicas são obrigatórias em 2024".to_string()
            ));
        }
        
        Ok(())
    }
} 