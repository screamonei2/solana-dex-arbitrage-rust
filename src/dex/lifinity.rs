use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;
use chrono::Utc;

use crate::dex::traits::{DexClient, Quote, DexError, DexType};
use crate::utils::constants::*;

/// Lifinity - Proactive Market Maker (PMM) 
/// Program ID: EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S
pub struct LifinityClient {
    rpc_client: reqwest::Client,
    rpc_url: String,
    program_id: Pubkey,
    api_base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LifinityPoolInfo {
    pub pool_id: String,
    pub token_a_mint: String,
    pub token_b_mint: String,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub fee_rate: f64,
    pub oracle_price: f64,
    pub liquidity_utilization: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LifinityQuote {
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub fee_amount: u64,
    pub oracle_price: f64,
    pub execution_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub slippage_tolerance: f64,
    pub user_public_key: String,
}

impl LifinityClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: reqwest::Client::new(),
            rpc_url,
            program_id: Pubkey::from_str("EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S")
                .expect("Invalid Lifinity program ID"),
            api_base_url: "https://api.lifinity.io".to_string(), // Placeholder
        }
    }

    /// Obter informações do pool BONK/SOL
    pub async fn get_pool_info(&self) -> Result<LifinityPoolInfo, DexError> {
        // Em produção, usar endpoint real da Lifinity
        let url = format!("{}/pools/bonk-sol", self.api_base_url);
        
        let response = self.rpc_client
            .get(&url)
            .send()
            .await
            .map_err(|e| DexError::Network(e))?;

        if !response.status().is_success() {
            return Err(DexError::Api {
                message: format!("Lifinity API error: {}", response.status()),
            });
        }

        // Simular dados do pool para desenvolvimento
        Ok(LifinityPoolInfo {
            pool_id: "lifinity_bonk_sol_pool".to_string(),
            token_a_mint: BONK_MINT.to_string(),
            token_b_mint: SOL_MINT.to_string(),
            token_a_reserve: 500_000_000_000, // 500B BONK
            token_b_reserve: 2_000_000_000,   // 2000 SOL
            fee_rate: 0.0008, // 0.08% base fee
            oracle_price: 0.000001, // 1 BONK = 0.000001 SOL
            liquidity_utilization: 0.65, // 65% utilização
        })
    }

    /// Calcular cotação usando Proactive Market Maker (PMM)
    pub async fn calculate_pmm_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
    ) -> Result<LifinityQuote, DexError> {
        let pool_info = self.get_pool_info().await?;
        
        // Determinar direção do swap
        let is_bonk_to_sol = input_mint == &*BONK_MINT_PUBKEY;
        
        // PMM formula considera:
        // 1. Oracle price (Pyth/Switchboard)
        // 2. Pool reserves
        // 3. Utilization rate
        // 4. Dynamic fee adjustment
        
        let oracle_price = pool_info.oracle_price;
        let utilization = pool_info.liquidity_utilization;
        
        // Ajustar fee baseado na utilização
        let dynamic_fee_multiplier = if utilization > 0.8 {
            1.5 // Aumentar fee se alta utilização
        } else if utilization < 0.3 {
            0.7 // Reduzir fee se baixa utilização
        } else {
            1.0
        };
        
        let adjusted_fee_rate = pool_info.fee_rate * dynamic_fee_multiplier;
        
        // Cálculo PMM simplificado
        let (output_amount, price_impact) = if is_bonk_to_sol {
            // BONK -> SOL
            let base_output = (amount as f64) * oracle_price;
            
            // Ajustar por price impact baseado no tamanho do trade
            let trade_size_ratio = (amount as f64) / (pool_info.token_a_reserve as f64);
            let impact_factor = (trade_size_ratio * 2.0).min(0.05); // Máximo 5% impact
            
            let adjusted_output = base_output * (1.0 - impact_factor);
            let final_output = adjusted_output * (1.0 - adjusted_fee_rate);
            
            (final_output as u64, impact_factor)
        } else {
            // SOL -> BONK
            let base_output = (amount as f64) / oracle_price;
            
            let trade_size_ratio = (amount as f64) / (pool_info.token_b_reserve as f64);
            let impact_factor = (trade_size_ratio * 2.0).min(0.05);
            
            let adjusted_output = base_output * (1.0 - impact_factor);
            let final_output = adjusted_output * (1.0 - adjusted_fee_rate);
            
            (final_output as u64, impact_factor)
        };
        
        let fee_amount = (amount as f64 * adjusted_fee_rate) as u64;
        let execution_price = if is_bonk_to_sol {
            (output_amount as f64) / (amount as f64)
        } else {
            (amount as f64) / (output_amount as f64)
        };

        Ok(LifinityQuote {
            input_amount: amount,
            output_amount,
            price_impact,
            fee_amount,
            oracle_price,
            execution_price,
        })
    }

    /// Criar instrução de swap
    pub async fn create_swap_transaction(
        &self,
        quote: &LifinityQuote,
        user_public_key: &Pubkey,
    ) -> Result<String, DexError> {
        // Em produção, usar Lifinity SDK para criar instruções reais
        let swap_request = SwapRequest {
            input_mint: BONK_MINT.to_string(), // Simplificado
            output_mint: SOL_MINT.to_string(),
            input_amount: quote.input_amount,
            slippage_tolerance: 0.5,
            user_public_key: user_public_key.to_string(),
        };

        let url = format!("{}/swap/create-transaction", self.api_base_url);
        
        let response = self.rpc_client
            .post(&url)
            .json(&swap_request)
            .send()
            .await
            .map_err(|e| DexError::Network(e))?;

        if !response.status().is_success() {
            return Err(DexError::Api {
                message: format!("Failed to create Lifinity swap transaction: {}", response.status()),
            });
        }

        // Placeholder - em produção retornaria transaction base64
        Ok("lifinity_swap_tx_placeholder".to_string())
    }
}

#[async_trait]
impl DexClient for LifinityClient {
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

        let pmm_quote = self.calculate_pmm_quote(input_mint, output_mint, amount).await?;
        
        // Verificar slippage tolerance
        let slippage_tolerance = slippage_bps as f64 / 10000.0;
        if pmm_quote.price_impact > slippage_tolerance {
            return Err(DexError::SlippageTooHigh {
                actual: pmm_quote.price_impact * 100.0,
                max: slippage_tolerance * 100.0,
            });
        }

        Ok(Quote {
            input_mint: *input_mint,
            output_mint: *output_mint,
            input_amount: amount,
            output_amount: pmm_quote.output_amount,
            price_impact: pmm_quote.price_impact,
            fees: pmm_quote.fee_amount,
            route: vec!["Lifinity".to_string(), "PMM".to_string()],
            dex: DexType::Lifinity,
            timestamp: Utc::now(),
        })
    }

    async fn execute_swap(
        &self,
        quote: &Quote,
        user_keypair: &Keypair,
    ) -> Result<String, DexError> {
        tracing::info!(
            "Executing Lifinity PMM swap: {} {} -> {} {} (impact: {:.2}%)",
            quote.input_amount,
            quote.input_mint,
            quote.output_amount,
            quote.output_mint,
            quote.price_impact * 100.0
        );

        // Simular execução - em produção usar Lifinity SDK
        let pmm_quote = LifinityQuote {
            input_amount: quote.input_amount,
            output_amount: quote.output_amount,
            price_impact: quote.price_impact,
            fee_amount: quote.fees,
            oracle_price: 0.000001, // Placeholder
            execution_price: (quote.output_amount as f64) / (quote.input_amount as f64),
        };

        let tx_signature = self.create_swap_transaction(&pmm_quote, &user_keypair.pubkey()).await?;
        
        tracing::info!("Lifinity PMM swap executed: {}", tx_signature);
        Ok(format!("lifinity_pmm_{}", chrono::Utc::now().timestamp()))
    }

    async fn get_liquidity(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<(u64, u64), DexError> {
        let is_bonk_sol = (input_mint == &*BONK_MINT_PUBKEY && output_mint == &*SOL_MINT_PUBKEY) ||
                          (input_mint == &*SOL_MINT_PUBKEY && output_mint == &*BONK_MINT_PUBKEY);

        if !is_bonk_sol {
            return Err(DexError::InvalidTokenPair {
                input: input_mint.to_string(),
                output: output_mint.to_string(),
            });
        }

        let pool_info = self.get_pool_info().await?;
        
        // Retornar reservas do pool
        Ok((pool_info.token_a_reserve, pool_info.token_b_reserve))
    }

    fn get_fee_bps(&self) -> u16 {
        80 // 0.08% base fee (pode variar dinamicamente)
    }

    fn get_dex_type(&self) -> DexType {
        DexType::Lifinity
    }

    async fn supports_pair(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<bool, DexError> {
        // Lifinity suporta BONK/SOL via PMM
        let is_bonk_sol = (input_mint == &*BONK_MINT_PUBKEY && output_mint == &*SOL_MINT_PUBKEY) ||
                          (input_mint == &*SOL_MINT_PUBKEY && output_mint == &*BONK_MINT_PUBKEY);
        Ok(is_bonk_sol)
    }

    fn get_name(&self) -> &'static str {
        "Lifinity"
    }
} 