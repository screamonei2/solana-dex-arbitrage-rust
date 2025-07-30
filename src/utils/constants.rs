use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;
use std::sync::OnceLock;
use once_cell::sync::Lazy;

// ============================================================================
// MINT ADDRESSES
// ============================================================================

/// BONK token mint address on Solana mainnet
pub const BONK_MINT_STRING: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";

/// SOL native token (wrapped SOL) mint address
pub const SOL_MINT_STRING: &str = "So11111111111111111111111111111111111111112";

/// USDC token mint address on Solana mainnet  
pub const USDC_MINT_STRING: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

// Lazy statics para Pubkeys (usando once_cell como alternativa a lazy_static)
static BONK_MINT_PUBKEY_CELL: OnceLock<Pubkey> = OnceLock::new();
static SOL_MINT_PUBKEY_CELL: OnceLock<Pubkey> = OnceLock::new();
static USDC_MINT_PUBKEY_CELL: OnceLock<Pubkey> = OnceLock::new();

/// Lazy initialization of BONK mint pubkey
pub fn get_bonk_mint_pubkey() -> &'static Pubkey {
    BONK_MINT_PUBKEY_CELL.get_or_init(|| {
        Pubkey::from_str(BONK_MINT_STRING).expect("Invalid BONK mint address")
    })
}

/// Lazy initialization of SOL mint pubkey  
pub fn get_sol_mint_pubkey() -> &'static Pubkey {
    SOL_MINT_PUBKEY_CELL.get_or_init(|| {
        Pubkey::from_str(SOL_MINT_STRING).expect("Invalid SOL mint address")
    })
}

/// Lazy initialization of USDC mint pubkey
pub fn get_usdc_mint_pubkey() -> &'static Pubkey {
    USDC_MINT_PUBKEY_CELL.get_or_init(|| {
        Pubkey::from_str(USDC_MINT_STRING).expect("Invalid USDC mint address")
    })
}

// Para compatibilidade com código existente, criamos referencias estáticas
// Estas serão inicializadas na primeira utilização
pub static BONK_MINT_PUBKEY: OnceLock<Pubkey> = OnceLock::new();
pub static SOL_MINT_PUBKEY: OnceLock<Pubkey> = OnceLock::new();
pub static USDC_MINT_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

/// Initialize all mint pubkeys - call this once at startup
pub fn initialize_mint_pubkeys() {
    let _ = BONK_MINT_PUBKEY.set(
        Pubkey::from_str(BONK_MINT_STRING).expect("Invalid BONK mint address")
    );
    let _ = SOL_MINT_PUBKEY.set(
        Pubkey::from_str(SOL_MINT_STRING).expect("Invalid SOL mint address")
    );
    let _ = USDC_MINT_PUBKEY.set(
        Pubkey::from_str(USDC_MINT_STRING).expect("Invalid USDC mint address")
    );
}

// ============================================================================
// DEX PROGRAM IDs
// ============================================================================

/// Raydium AMM Program ID
pub const RAYDIUM_AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Orca Whirlpool Program ID  
pub const ORCA_WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Jupiter Aggregator Program ID
pub const JUPITER_AGGREGATOR_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Meteora DLMM Program ID
pub const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";

/// Phoenix Program ID
pub const PHOENIX_PROGRAM_ID: &str = "PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY";

/// OpenBook Program ID  
pub const OPENBOOK_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";

/// Lifinity Program ID
pub const LIFINITY_PROGRAM_ID: &str = "EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S";

/// Saber Program ID
pub const SABER_PROGRAM_ID: &str = "SSwpkEEWHUBq56LUCrQNfWPiUKNbcCtC4Dk8wDaMHuP";

/// Mercurial Finance Program ID
pub const MERCURIAL_PROGRAM_ID: &str = "MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky";

/// Aldrin Program ID
pub const ALDRIN_PROGRAM_ID: &str = "CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4";

// ============================================================================
// RPC ENDPOINTS RECOMENDADOS (2024)
// ============================================================================

pub const RECOMMENDED_RPC_ENDPOINTS: &[&str] = &[
    "https://mainnet.helius-rpc.com",  // Helius (Professional)
    "https://rpc.ankr.com/solana",     // Ankr (Professional)  
    "https://solana-api.projectserum.com", // Serum (Backup)
    "https://api.mainnet-beta.solana.com", // Public (Limited)
];

// ============================================================================
// TRADING CONSTANTS
// ============================================================================

/// Minimum trade amount in lamports (0.001 SOL)
pub const MIN_TRADE_AMOUNT_LAMPORTS: u64 = 1_000_000;

/// Maximum trade amount in lamports (100 SOL)  
pub const MAX_TRADE_AMOUNT_LAMPORTS: u64 = 100_000_000_000;

/// Minimum profitable arbitrage in basis points (0.1%)
pub const MIN_PROFIT_BPS: u16 = 10;

/// Maximum slippage tolerance in basis points (0.5%)
pub const MAX_SLIPPAGE_BPS: u16 = 50;

/// Default trade timeout in seconds
pub const TRADE_TIMEOUT_SECONDS: u64 = 30;

// ============================================================================
// MEV PROTECTION CONSTANTS (2024 UPDATE)
// ============================================================================

/// Base priority fee in lamports per compute unit
pub const BASE_PRIORITY_FEE_LAMPORTS: u64 = 1_000;

/// Maximum priority fee in lamports per compute unit
pub const MAX_PRIORITY_FEE_LAMPORTS: u64 = 50_000;

/// Minimum Jito tip in lamports (if using Jito bundles)
pub const MIN_JITO_TIP_LAMPORTS: u64 = 10_000;

/// Maximum compute units for arbitrage transaction
pub const MAX_COMPUTE_UNITS: u32 = 1_400_000;

// ============================================================================
// RATE LIMITING CONSTANTS  
// ============================================================================

/// Default RPC requests per second for public endpoints
pub const DEFAULT_RPC_RATE_LIMIT: u32 = 10;

/// Professional RPC requests per second (Helius, QuickNode, etc)
pub const PROFESSIONAL_RPC_RATE_LIMIT: u32 = 500;

/// Maximum concurrent RPC requests
pub const MAX_CONCURRENT_REQUESTS: u32 = 20;

/// RPC request timeout in seconds
pub const RPC_TIMEOUT_SECONDS: u64 = 30;

// ============================================================================
// FAILOVER CONSTANTS
// ============================================================================

/// Maximum retries for failed RPC requests
pub const MAX_RPC_RETRIES: u32 = 3;

/// Failover threshold in milliseconds
pub const FAILOVER_THRESHOLD_MS: u64 = 5_000;

/// Circuit breaker failure threshold
pub const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;

/// Circuit breaker timeout in seconds
pub const CIRCUIT_BREAKER_TIMEOUT_SECONDS: u64 = 60;

// ============================================================================
// SECURITY CONSTANTS
// ============================================================================

/// Maximum loss tolerance in basis points (1%)
pub const MAX_LOSS_BPS: u16 = 100;

/// Position size limit as percentage of total balance (50%)
pub const MAX_POSITION_SIZE_PERCENT: f64 = 0.5;

/// Minimum account balance in lamports (0.1 SOL)
pub const MIN_ACCOUNT_BALANCE_LAMPORTS: u64 = 100_000_000;

// ============================================================================
// PERFORMANCE CONSTANTS
// ============================================================================

/// Maximum price data age in seconds (30 seconds)
pub const MAX_PRICE_AGE_SECONDS: u64 = 30;

/// Quote request timeout in milliseconds
pub const QUOTE_TIMEOUT_MS: u64 = 5_000;

/// WebSocket reconnection interval in seconds
pub const WEBSOCKET_RECONNECT_INTERVAL_SECONDS: u64 = 30;

// ============================================================================
// MONITORING CONSTANTS
// ============================================================================

/// Metrics update interval in seconds
pub const METRICS_UPDATE_INTERVAL_SECONDS: u64 = 10;

/// Health check interval in seconds  
pub const HEALTH_CHECK_INTERVAL_SECONDS: u64 = 60;

/// Log rotation interval in hours
pub const LOG_ROTATION_HOURS: u64 = 24;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Convert basis points to decimal (e.g., 100 bps = 0.01)
pub fn bps_to_decimal(bps: u16) -> f64 {
    bps as f64 / 10_000.0
}

/// Convert decimal to basis points (e.g., 0.01 = 100 bps)
pub fn decimal_to_bps(decimal: f64) -> u16 {
    (decimal * 10_000.0) as u16
}

/// Calculate lamports from SOL amount
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}

/// Calculate SOL amount from lamports
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

/// Check if amount is within trading limits
pub fn is_valid_trade_amount(lamports: u64) -> bool {
    lamports >= MIN_TRADE_AMOUNT_LAMPORTS && lamports <= MAX_TRADE_AMOUNT_LAMPORTS
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Calculate profit percentage from buy and sell prices
pub fn calculate_profit_percentage(buy_price: f64, sell_price: f64) -> f64 {
    if buy_price <= 0.0 {
        return 0.0;
    }
    ((sell_price - buy_price) / buy_price) * 100.0
} 

/// Initialize all static constants
pub fn init_constants() {
    let _ = BONK_MINT_PUBKEY.set(
        Pubkey::from_str(BONK_MINT_STRING).expect("Invalid BONK mint address")
    );
    let _ = SOL_MINT_PUBKEY.set(
        Pubkey::from_str(SOL_MINT_STRING).expect("Invalid SOL mint address")
    );
}

// Convenient aliases for common usage
pub static BONK_MINT: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str(BONK_MINT_STRING).expect("Invalid BONK mint address")
});

pub static SOL_MINT: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str(SOL_MINT_STRING).expect("Invalid SOL mint address")
}); 