use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::config::BotConfig;
use crate::monitoring::PriceData;
use crate::dex::DexType;
use crate::utils::{SOL_MINT_PUBKEY, BONK_MINT_PUBKEY};
use super::{ArbitrageOpportunity, StrategyType, TradeStep};

pub struct DirectArbitrageStrategy {
    config: Arc<BotConfig>,
    min_profit_bps: u16,
}

impl DirectArbitrageStrategy {
    pub fn new(config: Arc<BotConfig>) -> Self {
        let min_profit_bps = (config.trading.min_profit_threshold * 10000.0) as u16;
        
        Self {
            config,
            min_profit_bps,
        }
    }
    
    pub async fn find_opportunities(
        &self,
        prices: &HashMap<String, HashMap<DexType, PriceData>>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();
        
        // Check BONK/SOL arbitrage opportunities
        if let Some(bonk_sol_prices) = prices.get("BONK/SOL") {
            opportunities.extend(self.find_bonk_sol_opportunities(bonk_sol_prices).await);
        }
        
        // Check SOL/BONK arbitrage opportunities
        if let Some(sol_bonk_prices) = prices.get("SOL/BONK") {
            opportunities.extend(self.find_sol_bonk_opportunities(sol_bonk_prices).await);
        }
        
        opportunities
    }
    
    async fn find_bonk_sol_opportunities(
        &self,
        prices: &HashMap<DexType, PriceData>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();
        
        // Compare all DEX pairs for arbitrage
        for (dex_a, price_a) in prices {
            for (dex_b, price_b) in prices {
                if dex_a == dex_b {
                    continue;
                }
                
                // Calculate profit potential
                if let Some(opportunity) = self.calculate_arbitrage_opportunity(
                    *dex_a,
                    *dex_b,
                    price_a,
                    price_b,
                    *BONK_MINT_PUBKEY,
                    *SOL_MINT_PUBKEY,
                ).await {
                    opportunities.push(opportunity);
                }
            }
        }
        
        opportunities
    }
    
    async fn find_sol_bonk_opportunities(
        &self,
        prices: &HashMap<DexType, PriceData>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();
        
        // Compare all DEX pairs for arbitrage
        for (dex_a, price_a) in prices {
            for (dex_b, price_b) in prices {
                if dex_a == dex_b {
                    continue;
                }
                
                // Calculate profit potential
                if let Some(opportunity) = self.calculate_arbitrage_opportunity(
                    *dex_a,
                    *dex_b,
                    price_a,
                    price_b,
                    *SOL_MINT_PUBKEY,
                    *BONK_MINT_PUBKEY,
                ).await {
                    opportunities.push(opportunity);
                }
            }
        }
        
        opportunities
    }
    
    async fn calculate_arbitrage_opportunity(
        &self,
        dex_buy: DexType,
        dex_sell: DexType,
        price_buy: &PriceData,
        price_sell: &PriceData,
        input_mint: solana_sdk::pubkey::Pubkey,
        output_mint: solana_sdk::pubkey::Pubkey,
    ) -> Option<ArbitrageOpportunity> {
        // Basic arbitrage calculation: buy low, sell high
        if price_sell.price <= price_buy.price {
            return None;
        }
        
        let input_amount = self.calculate_optimal_amount(price_buy, price_sell)?;
        
        // Calculate expected output after fees and slippage
        let step1_output = self.calculate_swap_output(
            input_amount,
            price_buy.price,
            dex_buy,
            0.5, // 0.5% slippage estimate
        )?;
        
        let step2_output = self.calculate_swap_output(
            step1_output,
            price_sell.price,
            dex_sell,
            0.5, // 0.5% slippage estimate
        )?;
        
        // Calculate profit
        if step2_output <= input_amount {
            return None;
        }
        
        let profit = step2_output - input_amount;
        let profit_percentage = (profit as f64) / (input_amount as f64);
        let profit_bps = (profit_percentage * 10000.0) as u16;
        
        // Check if profit meets minimum threshold
        if profit_bps < self.min_profit_bps {
            return None;
        }
        
        // Check liquidity constraints
        if !self.check_liquidity_constraints(input_amount, price_buy, price_sell) {
            return None;
        }
        
        debug!(
            "Found direct arbitrage opportunity: {} -> {} -> {} profit: {}bps",
            dex_buy as u8, dex_sell as u8, dex_buy as u8, profit_bps
        );
        
        Some(ArbitrageOpportunity {
            strategy_type: StrategyType::Direct,
            input_amount,
            expected_output: step2_output,
            expected_profit: profit_percentage,
            profit_bps,
            path: vec![
                TradeStep {
                    dex: dex_buy,
                    input_mint,
                    output_mint,
                    amount_in: input_amount,
                    expected_amount_out: step1_output,
                    slippage_bps: 50, // 0.5%
                },
                TradeStep {
                    dex: dex_sell,
                    input_mint: output_mint,
                    output_mint: input_mint,
                    amount_in: step1_output,
                    expected_amount_out: step2_output,
                    slippage_bps: 50, // 0.5%
                },
            ],
            confidence: self.calculate_confidence(price_buy, price_sell),
            estimated_gas_cost: 10_000, // Estimate in lamports
        })
    }
    
    fn calculate_optimal_amount(
        &self,
        price_buy: &PriceData,
        price_sell: &PriceData,
    ) -> Option<u64> {
        // Use configured max position size as starting point
        let max_amount = self.config.trading.max_position_size;
        
        // Consider liquidity constraints
        let liquidity_limit = std::cmp::min(
            price_buy.liquidity.0 / 10, // Use 10% of available liquidity
            price_sell.liquidity.1 / 10,
        );
        
        // Use the smaller of max position size or liquidity limit
        Some(std::cmp::min(max_amount, liquidity_limit))
    }
    
    fn calculate_swap_output(
        &self,
        input_amount: u64,
        price: f64,
        dex: DexType,
        slippage_percentage: f64,
    ) -> Option<u64> {
        // Apply price
        let output_before_fees = (input_amount as f64 * price) as u64;
        
        // Apply DEX fees (simplified - in reality this would be DEX-specific)
        let fee_percentage = match dex {
            DexType::Raydium => 0.0025, // 0.25%
            DexType::Orca => 0.002,     // 0.2%
            DexType::Jupiter => 0.002,  // Depends on underlying DEX
            DexType::Meteora => 0.001,  // Variable, using average
            _ => 0.003,                 // Default 0.3%
        };
        
        let output_after_fees = output_before_fees as f64 * (1.0 - fee_percentage);
        
        // Apply slippage
        let final_output = output_after_fees * (1.0 - slippage_percentage / 100.0);
        
        Some(final_output as u64)
    }
    
    fn check_liquidity_constraints(
        &self,
        amount: u64,
        price_buy: &PriceData,
        price_sell: &PriceData,
    ) -> bool {
        // Check if there's sufficient liquidity for the trade
        let min_liquidity = amount * 5; // Need 5x the trade amount in liquidity
        
        price_buy.liquidity.0 >= min_liquidity && price_sell.liquidity.1 >= min_liquidity
    }
    
    fn calculate_confidence(
        &self,
        price_buy: &PriceData,
        price_sell: &PriceData,
    ) -> f64 {
        // Calculate confidence based on price age, liquidity, and spread
        let max_age_seconds = 30.0;
        let now = chrono::Utc::now();
        
        let age_buy = (now - price_buy.timestamp).num_seconds() as f64;
        let age_sell = (now - price_sell.timestamp).num_seconds() as f64;
        
        // Lower confidence for older prices
        let age_factor = (max_age_seconds - age_buy.max(age_sell)) / max_age_seconds;
        let age_factor = age_factor.max(0.0).min(1.0);
        
        // Higher confidence for higher liquidity
        let min_liquidity = 1_000_000u64; // 1M tokens
        let liquidity_factor = (price_buy.liquidity.0.min(price_sell.liquidity.1) as f64 / min_liquidity as f64).min(1.0);
        
        // Combine factors
        (age_factor * 0.6 + liquidity_factor * 0.4).max(0.1)
    }
} 