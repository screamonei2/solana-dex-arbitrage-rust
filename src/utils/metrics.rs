#[cfg(feature = "metrics")]
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    Opts, Registry, TextEncoder, Encoder
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use anyhow::Result;

use std::sync::OnceLock;

// Global metrics registry
pub static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    Registry::new()
});

// ============================================================================
// ARBITRAGE METRICS - Defined below with OnceLock
// ============================================================================

// ============================================================================
// METRICS INITIALIZATION
// ============================================================================

#[cfg(feature = "metrics")]
static PRICE_UPDATES_CELL: OnceLock<Counter> = OnceLock::new();
#[cfg(feature = "metrics")]
static OPPORTUNITIES_DETECTED_CELL: OnceLock<Counter> = OnceLock::new();

// Initialize metrics - call this once at startup
#[cfg(feature = "metrics")]
pub fn initialize_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let _ = PRICE_UPDATES_CELL.set(
        register_counter!("price_updates_total", "Total number of price updates received")?
    );
    
    let _ = ARBITRAGE_ATTEMPTS_CELL.set(
        register_counter!("arbitrage_attempts_total", "Total number of arbitrage attempts")?
    );
    
    let _ = ARBITRAGE_SUCCESSES_CELL.set(
        register_counter!("arbitrage_successes_total", "Total number of successful arbitrages")?
    );
    
    let _ = OPPORTUNITIES_DETECTED_CELL.set(
        register_counter!("opportunities_detected_total", "Total number of arbitrage opportunities detected")?
    );
    
    Ok(())
}

// ============================================================================
// METRIC ACCESSORS
// ============================================================================

#[cfg(feature = "metrics")]
pub fn get_price_updates() -> &'static Counter {
    PRICE_UPDATES_CELL.get().expect("Metrics not initialized")
}

#[cfg(feature = "metrics")]
pub fn get_arbitrage_attempts() -> &'static Counter {
    ARBITRAGE_ATTEMPTS_CELL.get().expect("Metrics not initialized")
}

#[cfg(feature = "metrics")]
pub fn get_arbitrage_successes() -> &'static Counter {
    ARBITRAGE_SUCCESSES_CELL.get().expect("Metrics not initialized")
}

#[cfg(feature = "metrics")]
pub fn get_opportunities_detected() -> &'static Counter {
    OPPORTUNITIES_DETECTED_CELL.get().expect("Metrics not initialized")
}

// ============================================================================
// COMPATIBILITY LAYER - For existing code
// ============================================================================

// Mock metrics for when prometheus feature is disabled
#[cfg(not(feature = "metrics"))]
pub struct MockCounter;

#[cfg(not(feature = "metrics"))]
impl MockCounter {
    pub fn inc(&self) {}
    pub fn inc_by(&self, _: u64) {}
}

#[cfg(not(feature = "metrics"))]
static MOCK_COUNTER: MockCounter = MockCounter;

// Export metrics with consistent naming
#[cfg(feature = "metrics")]
pub static PRICE_UPDATES: OnceLock<Counter> = OnceLock::new();
#[cfg(feature = "metrics")]
pub static ARBITRAGE_ATTEMPTS: OnceLock<Counter> = OnceLock::new();
#[cfg(feature = "metrics")]
pub static ARBITRAGE_SUCCESSES: OnceLock<Counter> = OnceLock::new();
#[cfg(feature = "metrics")]
pub static OPPORTUNITIES_DETECTED: OnceLock<Counter> = OnceLock::new();

#[cfg(not(feature = "metrics"))]
pub static PRICE_UPDATES: &MockCounter = &MOCK_COUNTER;
#[cfg(not(feature = "metrics"))]
pub static ARBITRAGE_ATTEMPTS: &MockCounter = &MOCK_COUNTER;
#[cfg(not(feature = "metrics"))]
pub static ARBITRAGE_SUCCESSES: &MockCounter = &MOCK_COUNTER;
#[cfg(not(feature = "metrics"))]
pub static OPPORTUNITIES_DETECTED: &MockCounter = &MOCK_COUNTER;

/// Initialize all metrics - call this once at startup
pub fn initialize() {
    #[cfg(feature = "metrics")]
    {
        let _ = initialize_metrics();
        
        // Set up static references for compatibility
        if let Some(counter) = PRICE_UPDATES_CELL.get() {
            let _ = PRICE_UPDATES.set(counter.clone());
        }
        if let Some(counter) = ARBITRAGE_ATTEMPTS_CELL.get() {
            let _ = ARBITRAGE_ATTEMPTS.set(counter.clone());
        }
        if let Some(counter) = ARBITRAGE_SUCCESSES_CELL.get() {
            let _ = ARBITRAGE_SUCCESSES.set(counter.clone());
        }
        if let Some(counter) = OPPORTUNITIES_DETECTED_CELL.get() {
            let _ = OPPORTUNITIES_DETECTED.set(counter.clone());
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Record a price update
pub fn record_price_update() {
    #[cfg(feature = "metrics")]
    if let Some(counter) = PRICE_UPDATES_CELL.get() {
        counter.inc();
    }
}

/// Record an arbitrage attempt
pub fn record_arbitrage_attempt() {
    #[cfg(feature = "metrics")]
    if let Some(counter) = ARBITRAGE_ATTEMPTS_CELL.get() {
        counter.inc();
    }
}

/// Record a successful arbitrage
pub fn record_arbitrage_success() {
    #[cfg(feature = "metrics")]
    if let Some(counter) = ARBITRAGE_SUCCESSES_CELL.get() {
        counter.inc();
    }
}

/// Record opportunities detected
pub fn record_opportunities_detected(count: u64) {
    #[cfg(feature = "metrics")]
    if let Some(counter) = OPPORTUNITIES_DETECTED_CELL.get() {
        counter.inc_by(count);
    }
} 