/// Calculate output amount for AMM swap using constant product formula
/// x * y = k
pub fn calculate_amm_output(
    pool_x: u64,
    pool_y: u64,
    amount_in: u64,
    fee_bps: u16,
) -> Option<u64> {
    if pool_x == 0 || pool_y == 0 || amount_in == 0 {
        return None;
    }
    
    // Apply fee
    let fee_decimal = fee_bps as f64 / 10000.0;
    let amount_in_after_fee = amount_in as f64 * (1.0 - fee_decimal);
    
    // Constant product formula: (x + Δx) * (y - Δy) = x * y
    // Solving for Δy: Δy = y * Δx / (x + Δx)
    let k = pool_x as f64 * pool_y as f64;
    let new_pool_x = pool_x as f64 + amount_in_after_fee;
    let new_pool_y = k / new_pool_x;
    
    let amount_out = pool_y as f64 - new_pool_y;
    
    if amount_out <= 0.0 {
        return None;
    }
    
    Some(amount_out as u64)
}

/// Calculate slippage for a given trade
pub fn calculate_slippage(
    pool_x: u64,
    pool_y: u64,
    amount_in: u64,
) -> f64 {
    if pool_x == 0 || pool_y == 0 || amount_in == 0 {
        return 0.0;
    }
    
    // Expected price (current pool ratio)
    let expected_price = pool_y as f64 / pool_x as f64;
    
    // Actual output after swap
    let amount_out = calculate_amm_output(pool_x, pool_y, amount_in, 0)
        .unwrap_or(0) as f64;
    
    if amount_out == 0.0 {
        return 100.0; // Maximum slippage
    }
    
    // Actual price from the trade
    let actual_price = amount_out / amount_in as f64;
    
    // Slippage percentage
    let slippage = (expected_price - actual_price) / expected_price;
    
    (slippage * 100.0).max(0.0)
}

/// Calculate price impact of a trade
pub fn calculate_price_impact(
    pool_x: u64,
    pool_y: u64,
    amount_in: u64,
) -> f64 {
    if pool_x == 0 || pool_y == 0 || amount_in == 0 {
        return 0.0;
    }
    
    let price_before = pool_y as f64 / pool_x as f64;
    
    // Simulate the trade
    let amount_out = calculate_amm_output(pool_x, pool_y, amount_in, 0)
        .unwrap_or(0);
    
    if amount_out == 0 {
        return 100.0;
    }
    
    let new_pool_x = pool_x + amount_in;
    let new_pool_y = pool_y - amount_out;
    
    let price_after = new_pool_y as f64 / new_pool_x as f64;
    
    let impact = (price_before - price_after) / price_before;
    
    (impact * 100.0).abs()
}

/// Calculate the optimal trade size to maximize profit
pub fn calculate_optimal_trade_size(
    pool_a_x: u64,
    pool_a_y: u64,
    pool_b_x: u64,
    pool_b_y: u64,
    fee_a_bps: u16,
    fee_b_bps: u16,
) -> Option<u64> {
    // Binary search for optimal amount
    let mut low = 1u64;
    let mut high = std::cmp::min(pool_a_x / 10, pool_b_y / 10); // Max 10% of pool
    let mut best_amount = 0u64;
    let mut best_profit = 0.0;
    
    while low <= high {
        let mid = (low + high) / 2;
        
        // Calculate profit for this amount
        let step1_output = calculate_amm_output(pool_a_x, pool_a_y, mid, fee_a_bps)?;
        let step2_output = calculate_amm_output(pool_b_y, pool_b_x, step1_output, fee_b_bps)?;
        
        let profit = step2_output as f64 - mid as f64;
        
        if profit > best_profit {
            best_profit = profit;
            best_amount = mid;
        }
        
        // Try larger amount if profitable
        if profit > 0.0 {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    
    if best_profit > 0.0 {
        Some(best_amount)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_amm_output_calculation() {
        // Pool with 100 SOL and 1M BONK
        let pool_sol = 100_000_000_000; // 100 SOL in lamports
        let pool_bonk = 1_000_000_000_000; // 1M BONK
        
        // Swap 1 SOL for BONK
        let amount_in = 1_000_000_000; // 1 SOL
        let output = calculate_amm_output(pool_sol, pool_bonk, amount_in, 25); // 0.25% fee
        
        assert!(output.is_some());
        assert!(output.unwrap() > 0);
    }
    
    #[test]
    fn test_slippage_calculation() {
        let pool_x = 100_000_000_000;
        let pool_y = 1_000_000_000_000;
        let amount_in = 1_000_000_000;
        
        let slippage = calculate_slippage(pool_x, pool_y, amount_in);
        assert!(slippage >= 0.0);
        assert!(slippage < 100.0);
    }
} 