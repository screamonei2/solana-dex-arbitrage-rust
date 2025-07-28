use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;

// ============================================================================
// RISK MANAGER STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_position_size: u64,
    pub max_slippage_bps: u16,
    pub max_price_impact_bps: u16,
    pub max_loss_per_trade_bps: u16,
    pub max_daily_loss_bps: u16,
    pub stop_loss_enabled: bool,
    pub circuit_breaker_enabled: bool,
    pub blacklisted_tokens: Vec<Pubkey>,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size: 1_000_000_000, // 1000 tokens
            max_slippage_bps: 100,             // 1%
            max_price_impact_bps: 200,         // 2%
            max_loss_per_trade_bps: 50,        // 0.5%
            max_daily_loss_bps: 500,           // 5%
            stop_loss_enabled: true,
            circuit_breaker_enabled: true,
            blacklisted_tokens: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_pnl: i64,
    pub daily_pnl: i64,
    pub max_drawdown: f64,
    pub win_rate: f64,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_pnl: 0,
            daily_pnl: 0,
            max_drawdown: 0.0,
            win_rate: 0.0,
        }
    }
}

// ============================================================================
// RISK MANAGER
// ============================================================================

pub struct RiskManager {
    config: RiskConfig,
    metrics: RiskMetrics,
    position_limits: HashMap<Pubkey, u64>,
    circuit_breaker_triggered: bool,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            metrics: RiskMetrics::default(),
            position_limits: HashMap::new(),
            circuit_breaker_triggered: false,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(RiskConfig::default())
    }

    // ========================================================================
    // VALIDATION METHODS
    // ========================================================================

    pub fn validate_trade_size(&self, amount: u64) -> Result<(), RiskError> {
        if amount > self.config.max_position_size {
            return Err(RiskError::PositionTooLarge {
                amount,
                max_allowed: self.config.max_position_size,
            });
        }
        Ok(())
    }

    pub fn validate_slippage(&self, slippage_bps: u16) -> Result<(), RiskError> {
        if slippage_bps > self.config.max_slippage_bps {
            return Err(RiskError::SlippageTooHigh {
                actual: slippage_bps,
                max_allowed: self.config.max_slippage_bps,
            });
        }
        Ok(())
    }

    pub fn validate_price_impact(&self, price_impact_bps: u16) -> Result<(), RiskError> {
        if price_impact_bps > self.config.max_price_impact_bps {
            return Err(RiskError::PriceImpactTooHigh {
                actual: price_impact_bps,
                max_allowed: self.config.max_price_impact_bps,
            });
        }
        Ok(())
    }

    pub fn validate_token(&self, token_mint: &Pubkey) -> Result<(), RiskError> {
        if self.config.blacklisted_tokens.contains(token_mint) {
            return Err(RiskError::BlacklistedToken {
                token: *token_mint,
            });
        }
        Ok(())
    }

    pub fn check_circuit_breaker(&self) -> Result<(), RiskError> {
        if self.circuit_breaker_triggered {
            return Err(RiskError::CircuitBreakerTriggered);
        }
        Ok(())
    }

    // ========================================================================
    // METRICS METHODS
    // ========================================================================

    pub fn record_trade(&mut self, success: bool, pnl: i64) {
        self.metrics.total_trades += 1;
        
        if success {
            self.metrics.successful_trades += 1;
        } else {
            self.metrics.failed_trades += 1;
        }

        self.metrics.total_pnl += pnl;
        self.metrics.daily_pnl += pnl;
        
        // Update win rate
        if self.metrics.total_trades > 0 {
            self.metrics.win_rate = 
                self.metrics.successful_trades as f64 / self.metrics.total_trades as f64;
        }

        // Check if circuit breaker should be triggered
        self.check_daily_loss_limit();
    }

    fn check_daily_loss_limit(&mut self) {
        if self.config.circuit_breaker_enabled {
            let daily_loss_bps = if self.metrics.daily_pnl < 0 {
                ((-self.metrics.daily_pnl) as f64 / 1_000_000.0 * 10_000.0) as u16
            } else {
                0
            };

            if daily_loss_bps > self.config.max_daily_loss_bps {
                self.circuit_breaker_triggered = true;
            }
        }
    }

    pub fn reset_daily_metrics(&mut self) {
        self.metrics.daily_pnl = 0;
        self.circuit_breaker_triggered = false;
    }

    pub fn get_metrics(&self) -> &RiskMetrics {
        &self.metrics
    }

    pub fn get_config(&self) -> &RiskConfig {
        &self.config
    }

    pub fn is_circuit_breaker_triggered(&self) -> bool {
        self.circuit_breaker_triggered
    }
}

// ============================================================================
// RISK ERRORS
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RiskError {
    #[error("Position size too large: {amount} > {max_allowed}")]
    PositionTooLarge { amount: u64, max_allowed: u64 },

    #[error("Slippage too high: {actual} bps > {max_allowed} bps")]
    SlippageTooHigh { actual: u16, max_allowed: u16 },

    #[error("Price impact too high: {actual} bps > {max_allowed} bps")]
    PriceImpactTooHigh { actual: u16, max_allowed: u16 },

    #[error("Token is blacklisted: {token}")]
    BlacklistedToken { token: Pubkey },

    #[error("Circuit breaker has been triggered")]
    CircuitBreakerTriggered,

    #[error("Daily loss limit exceeded")]
    DailyLossLimitExceeded,

    #[error("Trade validation failed: {message}")]
    ValidationError { message: String },
} 