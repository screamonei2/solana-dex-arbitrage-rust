pub mod direct;
pub mod triangular;
pub mod multi_dex;

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use crate::config::BotConfig;
use crate::monitoring::PriceData;
use crate::dex::DexType;

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub strategy_type: StrategyType,
    pub input_amount: u64,
    pub expected_output: u64,
    pub expected_profit: f64,
    pub profit_bps: u16,
    pub path: Vec<TradeStep>,
    pub confidence: f64, // 0.0 to 1.0
    pub estimated_gas_cost: u64,
}

#[derive(Debug, Clone)]
pub enum StrategyType {
    Direct,
    Triangular,
    MultiDex,
}

#[derive(Debug, Clone)]
pub struct TradeStep {
    pub dex: DexType,
    pub input_mint: solana_sdk::pubkey::Pubkey,
    pub output_mint: solana_sdk::pubkey::Pubkey,
    pub amount_in: u64,
    pub expected_amount_out: u64,
    pub slippage_bps: u16,
}

pub struct StrategyEngine {
    config: Arc<BotConfig>,
    direct_strategy: direct::DirectArbitrageStrategy,
    triangular_strategy: triangular::TriangularArbitrageStrategy,
    multi_dex_strategy: multi_dex::MultiDexArbitrageStrategy,
}

impl StrategyEngine {
    pub fn new(config: Arc<BotConfig>) -> Self {
        Self {
            direct_strategy: direct::DirectArbitrageStrategy::new(config.clone()),
            triangular_strategy: triangular::TriangularArbitrageStrategy::new(config.clone()),
            multi_dex_strategy: multi_dex::MultiDexArbitrageStrategy::new(config.clone()),
            config,
        }
    }
    
    pub async fn find_opportunities(
        &self,
        prices: &HashMap<String, HashMap<DexType, PriceData>>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();
        
        // Run all strategies in parallel
        let direct_futures = self.direct_strategy.find_opportunities(prices);
        let triangular_futures = self.triangular_strategy.find_opportunities(prices);
        let multi_dex_futures = self.multi_dex_strategy.find_opportunities(prices);
        
        let (direct_ops, triangular_ops, multi_dex_ops) = tokio::join!(
            direct_futures,
            triangular_futures,
            multi_dex_futures
        );
        
        opportunities.extend(direct_ops);
        opportunities.extend(triangular_ops);
        opportunities.extend(multi_dex_ops);
        
        // Sort by profit potential
        opportunities.sort_by(|a, b| {
            b.expected_profit.partial_cmp(&a.expected_profit).unwrap()
        });
        
        // Filter by minimum profit threshold
        opportunities.retain(|op| {
            op.expected_profit >= self.config.trading.min_profit_threshold
        });
        
        crate::utils::metrics::OPPORTUNITIES_DETECTED.inc_by(opportunities.len() as u64);
        
        opportunities
    }
} 