use crate::math::amm::*;

/// Calculate profit from a two-step arbitrage
pub fn calculate_arbitrage_profit(
    input_amount: u64,
    pool1_x: u64,
    pool1_y: u64,
    fee1_bps: u16,
    pool2_x: u64,
    pool2_y: u64,
    fee2_bps: u16,
) -> Option<i64> {
    // Step 1: Swap in first pool
    let intermediate_amount = calculate_amm_output(pool1_x, pool1_y, input_amount, fee1_bps)?;
    
    // Step 2: Swap in second pool
    let final_amount = calculate_amm_output(pool2_x, pool2_y, intermediate_amount, fee2_bps)?;
    
    // Calculate profit (can be negative)
    Some(final_amount as i64 - input_amount as i64)
}

/// Calculate optimal trade size for maximum profit
pub fn find_optimal_arbitrage_amount(
    pool1_x: u64,
    pool1_y: u64,
    fee1_bps: u16,
    pool2_x: u64,
    pool2_y: u64,
    fee2_bps: u16,
    max_amount: u64,
) -> Option<u64> {
    let mut best_amount = 0u64;
    let mut best_profit = 0i64;
    
    // Binary search for optimal amount
    let mut low = 1u64;
    let mut high = max_amount;
    
    while low <= high {
        let mid = (low + high) / 2;
        
        if let Some(profit) = calculate_arbitrage_profit(mid, pool1_x, pool1_y, fee1_bps, pool2_x, pool2_y, fee2_bps) {
            if profit > best_profit {
                best_profit = profit;
                best_amount = mid;
            }
            
            // Check derivative direction
            if profit > 0 {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        } else {
            high = mid - 1;
        }
    }
    
    if best_profit > 0 {
        Some(best_amount)
    } else {
        None
    }
}

/// Calculate expected profit percentage
pub fn calculate_profit_percentage(
    input_amount: u64,
    final_amount: u64,
) -> f64 {
    if input_amount == 0 {
        return 0.0;
    }
    
    ((final_amount as f64 - input_amount as f64) / input_amount as f64) * 100.0
}

/// Check if arbitrage is profitable after all costs
pub fn is_arbitrage_profitable(
    input_amount: u64,
    final_amount: u64,
    gas_cost: u64,
    min_profit_bps: u16,
) -> bool {
    if final_amount <= input_amount + gas_cost {
        return false;
    }
    
    let net_profit = final_amount - input_amount - gas_cost;
    let profit_bps = (net_profit as f64 / input_amount as f64 * 10000.0) as u16;
    
    profit_bps >= min_profit_bps
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arbitrage_profit_calculation() {
        // Setup two pools with price difference
        let pool1_x = 100_000_000_000; // 100 SOL
        let pool1_y = 1_000_000_000_000; // 1M BONK
        let pool2_x = 500_000_000_000; // 500 BONK  
        let pool2_y = 50_000_000_000; // 50 SOL
        
        let input_amount = 1_000_000_000; // 1 SOL
        
        let profit = calculate_arbitrage_profit(
            input_amount,
            pool1_x, pool1_y, 25, // 0.25% fee
            pool2_x, pool2_y, 25, // 0.25% fee
        );
        
        assert!(profit.is_some());
    }
    
    #[test]
    fn test_profit_percentage() {
        let percentage = calculate_profit_percentage(1000, 1050);
        assert_eq!(percentage, 5.0); // 5% profit
    }
} 