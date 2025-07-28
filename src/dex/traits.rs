use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use solana_sdk::transaction::Transaction;
use std::collections::HashMap;

// ============================================================================
// DEX ERROR TYPE
// ============================================================================

#[derive(Debug, Error)]
pub enum DexError {
    #[error("Failed to fetch quote: {0}")]
    QuoteFetchError(String),
    
    #[error("Insufficient liquidity for trade")]
    InsufficientLiquidity,
    
    #[error("Price impact too high: {0}%")]
    PriceImpactTooHigh(f64),
    
    #[error("Pool not found: {0}")]
    PoolNotFound(String),
    
    #[error("Invalid pool state")]
    InvalidPoolState,
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Invalid token pair")]
    InvalidTokenPair,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// ============================================================================
// DEX TYPE ENUM
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexType {
    Raydium,
    Orca,
    Jupiter,
    Meteora,
    Phoenix,
    OpenBook,
    Lifinity,
    Saber,
    Mercurial,
    Aldrin,
}

impl DexType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DexType::Raydium => "raydium",
            DexType::Orca => "orca", 
            DexType::Jupiter => "jupiter",
            DexType::Meteora => "meteora",
            DexType::Phoenix => "phoenix",
            DexType::OpenBook => "openbook",
            DexType::Lifinity => "lifinity",
            DexType::Saber => "saber",
            DexType::Mercurial => "mercurial",
            DexType::Aldrin => "aldrin",
        }
    }
}

// ============================================================================
// QUOTE STRUCTURE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub dex_type: DexType,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub fees: u64,
    pub slippage_bps: u16,
    pub route: Option<Vec<Pubkey>>, // For multi-hop swaps
    pub valid_until: Option<u64>,   // Timestamp in seconds
}

// ============================================================================
// SWAP RESULT STRUCTURE  
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub signature: String,
    pub success: bool,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fees_paid: u64,
    pub execution_time_ms: u64,
}

// ============================================================================
// DEX CLIENT TRAIT
// ============================================================================

#[async_trait]
pub trait DexClient: Send + Sync {
    /// Get the DEX type
    fn dex_type(&self) -> DexType;
    
    /// Get the DEX type (alias for compatibility)
    fn get_dex_type(&self) -> DexType {
        self.dex_type()
    }
    
    /// Get the DEX name
    fn get_name(&self) -> &'static str;
    
    /// Get fee in basis points
    fn get_fee_bps(&self) -> u16;
    
    /// Get liquidity for a trading pair (returns token_a_amount, token_b_amount)
    async fn get_liquidity(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
    ) -> Result<(u64, u64), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get a quote for swapping tokens
    async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Quote, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Execute a swap transaction
    async fn execute_swap(
        &self,
        quote: &Quote,
        user_keypair: &solana_sdk::signature::Keypair,
    ) -> Result<SwapResult, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get current fee information
    async fn get_fee_info(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Check if a token pair is supported
    async fn supports_pair(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
} 