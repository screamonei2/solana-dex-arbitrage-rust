use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;
use super::traits::{DexClient, DexError, DexType, Quote};

pub struct PhoenixClient;
impl PhoenixClient { pub fn new() -> Self { Self } }
#[async_trait]
impl DexClient for PhoenixClient {
    async fn get_quote(&self, _: &Pubkey, _: &Pubkey, _: u64, _: u16) -> Result<Quote, DexError> { 
        Err(DexError::Api { message: "Not implemented".to_string() }) 
    }
    async fn execute_swap(&self, _: &Quote, _: &solana_sdk::signature::Keypair) -> Result<String, DexError> { 
        Err(DexError::Api { message: "Not implemented".to_string() }) 
    }
    async fn get_liquidity(&self, _: &Pubkey, _: &Pubkey) -> Result<(u64, u64), DexError> { Ok((0, 0)) }
    fn get_fee_bps(&self) -> u16 { 0 }
    fn get_dex_type(&self) -> DexType { DexType::Phoenix }
    async fn supports_pair(&self, _: &Pubkey, _: &Pubkey) -> Result<bool, DexError> { Ok(false) }
    fn get_name(&self) -> &'static str { "Phoenix" }
} 