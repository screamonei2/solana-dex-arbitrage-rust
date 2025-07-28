pub mod manager;

pub use manager::{RiskManager, RiskConfig, RiskMetrics, RiskError};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::config::BotConfig;
use crate::strategy::ArbitrageOpportunity;

#[derive(Debug)]
pub struct RiskMetrics {
    pub daily_loss: f64,
    pub consecutive_failures: u32,
    pub last_failure_time: Option<DateTime<Utc>>,
    pub total_trades: u64,
    pub successful_trades: u64,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            daily_loss: 0.0,
            consecutive_failures: 0,
            last_failure_time: None,
            total_trades: 0,
            successful_trades: 0,
        }
    }
}

pub struct RiskManager {
    config: Arc<BotConfig>,
    metrics: Arc<RwLock<RiskMetrics>>,
}

impl RiskManager {
    pub fn new(config: Arc<BotConfig>) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(RiskMetrics::default())),
        }
    }
    
    pub async fn validate_opportunity(&self, opportunity: &ArbitrageOpportunity) -> bool {
        let metrics = self.metrics.read().await;
        
        // Check if circuit breaker is enabled and triggered
        if self.config.risk.circuit_breaker_enabled {
            // Check daily loss limit
            if metrics.daily_loss > self.config.risk.max_daily_loss {
                tracing::warn!("Daily loss limit exceeded: {:.2}%", metrics.daily_loss * 100.0);
                return false;
            }
            
            // Check consecutive failures
            if metrics.consecutive_failures >= self.config.risk.max_consecutive_failures {
                tracing::warn!("Too many consecutive failures: {}", metrics.consecutive_failures);
                return false;
            }
        }
        
        // Check position size
        if opportunity.input_amount > self.config.trading.max_position_size {
            tracing::warn!("Position size too large: {} > {}", 
                opportunity.input_amount, 
                self.config.trading.max_position_size
            );
            return false;
        }
        
        // Check minimum profit threshold
        if opportunity.expected_profit < self.config.trading.min_profit_threshold {
            return false;
        }
        
        // Check confidence level
        if opportunity.confidence < 0.7 {
            tracing::debug!("Opportunity confidence too low: {:.2}", opportunity.confidence);
            return false;
        }
        
        true
    }
    
    pub async fn record_trade_result(&self, profit: f64, success: bool) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_trades += 1;
        
        if success {
            metrics.successful_trades += 1;
            metrics.consecutive_failures = 0;
        } else {
            metrics.consecutive_failures += 1;
            metrics.last_failure_time = Some(Utc::now());
        }
        
        // Update daily loss (negative profit)
        if profit < 0.0 {
            metrics.daily_loss += profit.abs();
        }
        
        tracing::info!(
            "Trade recorded: profit={:.4} success={} consecutive_failures={}",
            profit,
            success,
            metrics.consecutive_failures
        );
    }
    
    pub async fn get_metrics(&self) -> RiskMetrics {
        self.metrics.read().await.clone()
    }
    
    pub async fn reset_daily_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.daily_loss = 0.0;
        tracing::info!("Daily risk metrics reset");
    }
} 