use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use anyhow::Result;

/// Convert lamports to SOL
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

/// Convert SOL to lamports
pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}

/// Parse Pubkey from string safely
pub fn parse_pubkey(key_str: &str) -> Result<Pubkey> {
    Pubkey::from_str(key_str)
        .map_err(|e| anyhow::anyhow!("Invalid pubkey '{}': {}", key_str, e))
}

/// Calculate percentage change
pub fn percentage_change(old_value: f64, new_value: f64) -> f64 {
    if old_value == 0.0 {
        return 0.0;
    }
    ((new_value - old_value) / old_value) * 100.0
}

/// Calculate basis points
pub fn to_basis_points(percentage: f64) -> u16 {
    (percentage * 100.0) as u16
}

/// Convert basis points to percentage
pub fn from_basis_points(bps: u16) -> f64 {
    bps as f64 / 100.0
}

/// Safe math operations to prevent overflow
pub fn safe_add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| anyhow::anyhow!("Overflow in addition: {} + {}", a, b))
}

pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b)
        .ok_or_else(|| anyhow::anyhow!("Underflow in subtraction: {} - {}", a, b))
}

pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b)
        .ok_or_else(|| anyhow::anyhow!("Overflow in multiplication: {} * {}", a, b))
}

/// Format transaction signature for display
pub fn format_signature(signature: &str) -> String {
    if signature.len() > 12 {
        format!("{}...{}", &signature[..6], &signature[signature.len()-6..])
    } else {
        signature.to_string()
    }
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
} 