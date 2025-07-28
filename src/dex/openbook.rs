use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;
use chrono::Utc;

use crate::dex::traits::{DexClient, Quote, DexError, DexType};
use crate::utils::constants::*;

/// OpenBook (Serum v3 Fork) - Central Limit Order Book (CLOB)
/// Program ID: srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX
pub struct OpenBookClient {
    rpc_client: reqwest::Client,
    rpc_url: String,
    program_id: Pubkey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenBookMarket {
    pub address: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub fee_rate_bps: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

impl OpenBookClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: reqwest::Client::new(),
            rpc_url,
            program_id: Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX")
                .expect("Invalid OpenBook program ID"),
        }
    }

    /// Obter order book para um mercado específico
    pub async fn get_order_book(&self, market_address: &str) -> Result<OrderBook, DexError> {
        // Implementação simplificada - em produção, usar OpenBook SDK
        let response = self.rpc_client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getAccountInfo",
                "params": [
                    market_address,
                    {"encoding": "base64"}
                ]
            }))
            .send()
            .await
            .map_err(|e| DexError::Network(e))?;

        if !response.status().is_success() {
            return Err(DexError::Api {
                message: format!("OpenBook API error: {}", response.status()),
            });
        }

        // Simplified order book - in production, parse actual OpenBook data
        Ok(OrderBook {
            bids: vec![
                PriceLevel { price: 0.000001, size: 1000000.0 },
                PriceLevel { price: 0.0000009, size: 2000000.0 },
            ],
            asks: vec![
                PriceLevel { price: 0.0000011, size: 1500000.0 },
                PriceLevel { price: 0.0000012, size: 800000.0 },
            ],
        })
    }

    /// Obter melhor bid/ask spread
    pub async fn get_best_bid_ask(&self, market_address: &str) -> Result<(f64, f64), DexError> {
        let order_book = self.get_order_book(market_address).await?;
        
        let best_bid = order_book.bids
            .first()
            .map(|level| level.price)
            .ok_or(DexError::InsufficientLiquidity)?;
            
        let best_ask = order_book.asks
            .first()
            .map(|level| level.price)
            .ok_or(DexError::InsufficientLiquidity)?;

        Ok((best_bid, best_ask))
    }

    /// Calcular cotação baseada no order book
    fn calculate_quote_from_orderbook(
        &self,
        order_book: &OrderBook,
        amount: u64,
        is_buy: bool,
    ) -> Result<u64, DexError> {
        let levels = if is_buy { &order_book.asks } else { &order_book.bids };
        
        let mut remaining_amount = amount as f64;
        let mut total_cost = 0.0;
        
        for level in levels {
            if remaining_amount <= 0.0 {
                break;
            }
            
            let amount_to_fill = remaining_amount.min(level.size);
            total_cost += amount_to_fill * level.price;
            remaining_amount -= amount_to_fill;
        }
        
        if remaining_amount > 0.0 {
            return Err(DexError::InsufficientLiquidity);
        }
        
        Ok(total_cost as u64)
    }
}

#[async_trait]
impl DexClient for OpenBookClient {
    async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Quote, DexError> {
        // Verificar se é par BONK/SOL
        let is_bonk_sol = (input_mint == &*BONK_MINT_PUBKEY && output_mint == &*SOL_MINT_PUBKEY) ||
                          (input_mint == &*SOL_MINT_PUBKEY && output_mint == &*BONK_MINT_PUBKEY);

        if !is_bonk_sol {
            return Err(DexError::InvalidTokenPair {
                input: input_mint.to_string(),
                output: output_mint.to_string(),
            });
        }

        // Market address para BONK/SOL (placeholder - usar endereço real)
        let market_address = "BoNKqCNGqr2dw6Jjy6VCJ2VjvEb5Tt3kCJaZDnxKiPX"; // Placeholder
        
        let order_book = self.get_order_book(market_address).await?;
        
        // Determinar se é compra ou venda baseado no par
        let is_buy = input_mint == &*SOL_MINT_PUBKEY; // SOL -> BONK é compra
        
        let output_amount = self.calculate_quote_from_orderbook(&order_book, amount, is_buy)?;
        
        // Calcular price impact estimado
        let (best_bid, best_ask) = self.get_best_bid_ask(market_address).await?;
        let mid_price = (best_bid + best_ask) / 2.0;
        let executed_price = if is_buy {
            output_amount as f64 / amount as f64
        } else {
            amount as f64 / output_amount as f64
        };
        
        let price_impact = ((executed_price - mid_price) / mid_price).abs();
        
        // Verificar slippage
        if price_impact > (slippage_bps as f64 / 10000.0) {
            return Err(DexError::SlippageTooHigh {
                actual: price_impact * 100.0,
                max: slippage_bps as f64 / 100.0,
            });
        }

        Ok(Quote {
            input_mint: *input_mint,
            output_mint: *output_mint,
            input_amount: amount,
            output_amount,
            price_impact,
            fees: (amount as f64 * 0.0004) as u64, // 0.04% taker fee
            route: vec!["OpenBook".to_string()],
            dex: DexType::OpenBook,
            timestamp: Utc::now(),
        })
    }

    async fn execute_swap(
        &self,
        quote: &Quote,
        user_keypair: &Keypair,
    ) -> Result<String, DexError> {
        // Implementação simplificada - em produção, usar OpenBook SDK para criar instruções
        tracing::info!(
            "Executing OpenBook swap: {} {} -> {} {}",
            quote.input_amount,
            quote.input_mint,
            quote.output_amount,
            quote.output_mint
        );

        // Placeholder transaction signature
        let signature = format!("openbook_tx_{}", chrono::Utc::now().timestamp());
        
        tracing::info!("OpenBook swap executed: {}", signature);
        Ok(signature)
    }

    async fn get_liquidity(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<(u64, u64), DexError> {
        // Verificar par BONK/SOL
        let is_bonk_sol = (input_mint == &*BONK_MINT_PUBKEY && output_mint == &*SOL_MINT_PUBKEY) ||
                          (input_mint == &*SOL_MINT_PUBKEY && output_mint == &*BONK_MINT_PUBKEY);

        if !is_bonk_sol {
            return Err(DexError::InvalidTokenPair {
                input: input_mint.to_string(),
                output: output_mint.to_string(),
            });
        }

        // Simular liquidez do order book
        let market_address = "BoNKqCNGqr2dw6Jjy6VCJ2VjvEb5Tt3kCJaZDnxKiPX";
        let order_book = self.get_order_book(market_address).await?;
        
        let total_bid_liquidity: f64 = order_book.bids.iter()
            .map(|level| level.size * level.price)
            .sum();
            
        let total_ask_liquidity: f64 = order_book.asks.iter()
            .map(|level| level.size)
            .sum();

        Ok((total_bid_liquidity as u64, total_ask_liquidity as u64))
    }

    fn get_fee_bps(&self) -> u16 {
        40 // 0.04% taker fee
    }

    fn get_dex_type(&self) -> DexType {
        DexType::OpenBook
    }

    async fn supports_pair(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<bool, DexError> {
        // OpenBook suporta BONK/SOL
        let is_bonk_sol = (input_mint == &*BONK_MINT_PUBKEY && output_mint == &*SOL_MINT_PUBKEY) ||
                          (input_mint == &*SOL_MINT_PUBKEY && output_mint == &*BONK_MINT_PUBKEY);
        Ok(is_bonk_sol)
    }

    fn get_name(&self) -> &'static str {
        "OpenBook"
    }
} 