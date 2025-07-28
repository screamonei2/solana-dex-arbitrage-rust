use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn, info};
use thiserror::Error;

use crate::utils::constants::*;

#[derive(Debug, Error)]
pub enum PriorityFeeError {
    #[error("Failed to get recent prioritization fees: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),
    
    #[error("Invalid fee calculation: {message}")]
    InvalidCalculation { message: String },
    
    #[error("Fee exceeds maximum: {fee} > {max}")]
    ExceedsMaximum { fee: u64, max: u64 },
    
    #[error("Network congestion data unavailable")]
    NoCongestionData,
}

/// CRÍTICO 2024: Calculadora de priority fees dinâmicas
/// Substitui MEV protection após suspensão do Jito mempool público
#[derive(Clone)]
pub struct DynamicPriorityFeeCalculator {
    rpc_client: Arc<RpcClient>,
    base_fee: u64,
    max_fee: u64,
    min_fee: u64,
    congestion_history: Arc<RwLock<Vec<CongestionSample>>>,
    last_update: Arc<RwLock<Instant>>,
    update_interval: Duration,
}

#[derive(Debug, Clone)]
struct CongestionSample {
    timestamp: Instant,
    average_fee: u64,
    median_fee: u64,
    p75_fee: u64,
    p95_fee: u64,
    slot: u64,
    transaction_count: u32,
}

#[derive(Debug, Clone)]
pub struct PriorityFeeRecommendation {
    pub recommended_fee: u64,
    pub confidence_level: f64, // 0.0 - 1.0
    pub reasoning: String,
    pub congestion_level: CongestionLevel,
    pub estimated_confirmation_time: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CongestionLevel {
    Low,       // < 25% capacity
    Medium,    // 25-75% capacity
    High,      // 75-90% capacity
    Critical,  // > 90% capacity
}

impl DynamicPriorityFeeCalculator {
    /// Criar novo calculador de priority fees dinâmicas
    pub fn new(rpc_client: Arc<RpcClient>, config: crate::execution::jito::MevProtectionConfig) -> Self {
        Self {
            rpc_client,
            base_fee: MIN_PRIORITY_FEE_LAMPORTS,
            max_fee: config.max_priority_fee_lamports,
            min_fee: MIN_PRIORITY_FEE_LAMPORTS,
            congestion_history: Arc::new(RwLock::new(Vec::new())),
            last_update: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(3600))),
            update_interval: Duration::from_secs(30), // Atualizar a cada 30 segundos
        }
    }
    
    /// CRÍTICO 2024: Obter priority fee recomendado (alias para compatibilidade)
    pub async fn get_recommended_priority_fee(&self) -> Result<u64, PriorityFeeError> {
        let recommendation = self.get_recommended_fee().await?;
        Ok(recommendation.recommended_fee)
    }
    
    /// CRÍTICO 2024: Obter priority fee recomendado baseado em congestionamento atual
    pub async fn get_recommended_fee(&self) -> Result<PriorityFeeRecommendation, PriorityFeeError> {
        // Atualizar dados de congestionamento se necessário
        self.update_congestion_data().await?;
        
        let congestion_history = self.congestion_history.read().await;
        
        if congestion_history.is_empty() {
            warn!("Sem dados de congestionamento, usando fee base");
            return Ok(PriorityFeeRecommendation {
                recommended_fee: self.base_fee,
                confidence_level: 0.3,
                reasoning: "Sem dados históricos, usando fee base".to_string(),
                congestion_level: CongestionLevel::Medium,
                estimated_confirmation_time: Duration::from_secs(30),
            });
        }
        
        // Analisar tendência recente (últimos 5 samples)
        let recent_samples: Vec<_> = congestion_history
            .iter()
            .rev()
            .take(5)
            .collect();
        
        let avg_recent_fee = recent_samples.iter()
            .map(|s| s.p75_fee)
            .sum::<u64>() / recent_samples.len() as u64;
        
        let avg_transaction_count = recent_samples.iter()
            .map(|s| s.transaction_count)
            .sum::<u32>() / recent_samples.len() as u32;
        
        // Calcular nível de congestionamento
        let congestion_level = self.calculate_congestion_level(avg_transaction_count);
        
        // Calcular fee recomendado baseado no congestionamento
        let recommended_fee = self.calculate_dynamic_fee(
            avg_recent_fee,
            &congestion_level,
            recent_samples.len(),
        )?;
        
        // Calcular confiança baseada na quantidade de dados
        let confidence_level = (recent_samples.len() as f64 / 5.0).min(1.0);
        
        // Estimar tempo de confirmação
        let estimated_confirmation_time = self.estimate_confirmation_time(&congestion_level);
        
        let reasoning = format!(
            "Baseado em {} samples recentes. Fee médio p75: {} lamports. Congestionamento: {:?}",
            recent_samples.len(),
            avg_recent_fee,
            congestion_level
        );
        
        info!(
            "Priority fee recomendado: {} lamports (congestionamento: {:?})",
            recommended_fee, congestion_level
        );
        
        Ok(PriorityFeeRecommendation {
            recommended_fee,
            confidence_level,
            reasoning,
            congestion_level,
            estimated_confirmation_time,
        })
    }
    
    /// Atualizar dados de congestionamento da rede
    async fn update_congestion_data(&self) -> Result<(), PriorityFeeError> {
        let mut last_update = self.last_update.write().await;
        
        // Verificar se precisa atualizar
        if last_update.elapsed() < self.update_interval {
            return Ok(());
        }
        
        debug!("Atualizando dados de congestionamento da rede");
        
        // Obter fees recentes (últimos 150 slots)
        let recent_fees = self.rpc_client
            .get_recent_prioritization_fees(&[])
            .await?;
        
        if recent_fees.is_empty() {
            return Err(PriorityFeeError::NoCongestionData);
        }
        
        // Processar dados por slot
        let mut slot_data: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
        
        for fee_data in recent_fees {
            slot_data
                .entry(fee_data.slot)
                .or_insert_with(Vec::new)
                .push(fee_data.prioritization_fee);
        }
        
        // Calcular estatísticas por slot
        let mut congestion_history = self.congestion_history.write().await;
        
        for (slot, mut fees) in slot_data {
            fees.sort();
            
            if fees.is_empty() {
                continue;
            }
            
            let len = fees.len();
            let average_fee = fees.iter().sum::<u64>() / len as u64;
            let median_fee = fees[len / 2];
            let p75_fee = fees[(len * 3) / 4];
            let p95_fee = fees[(len * 95) / 100];
            
            let sample = CongestionSample {
                timestamp: Instant::now(),
                average_fee,
                median_fee,
                p75_fee,
                p95_fee,
                slot,
                transaction_count: len as u32,
            };
            
            congestion_history.push(sample);
        }
        
        // Manter apenas últimos 100 samples
        if congestion_history.len() > 100 {
            congestion_history.drain(0..congestion_history.len() - 100);
        }
        
        *last_update = Instant::now();
        
        debug!(
            "Dados de congestionamento atualizados. Samples: {}",
            congestion_history.len()
        );
        
        Ok(())
    }
    
    /// Calcular fee dinâmico baseado em congestionamento
    fn calculate_dynamic_fee(
        &self,
        recent_avg_fee: u64,
        congestion_level: &CongestionLevel,
        confidence_samples: usize,
    ) -> Result<u64, PriorityFeeError> {
        let base_multiplier = match congestion_level {
            CongestionLevel::Low => 1.0,
            CongestionLevel::Medium => 1.5,
            CongestionLevel::High => 2.0,
            CongestionLevel::Critical => 3.0,
        };
        
        // Ajustar baseado na confiança dos dados
        let confidence_multiplier = if confidence_samples >= 3 {
            1.0
        } else {
            1.2 // Ser mais conservador com poucos dados
        };
        
        // Calcular fee baseado em dados recentes ou fee base
        let calculated_fee = if recent_avg_fee > 0 {
            // Usar fee da rede como base, aplicar multiplicadores
            let network_based_fee = (recent_avg_fee as f64 * base_multiplier * confidence_multiplier) as u64;
            
            // Garantir que não seja muito abaixo do fee base
            network_based_fee.max(self.base_fee)
        } else {
            // Fallback para fee base com multiplicador
            (self.base_fee as f64 * base_multiplier * confidence_multiplier) as u64
        };
        
        // Aplicar limites
        let final_fee = calculated_fee
            .max(self.min_fee)
            .min(self.max_fee);
        
        if final_fee == self.max_fee && calculated_fee > self.max_fee {
            warn!(
                "Priority fee calculado ({}) excede máximo ({}), limitando",
                calculated_fee, self.max_fee
            );
        }
        
        Ok(final_fee)
    }
    
    /// Calcular nível de congestionamento baseado em transaction count
    fn calculate_congestion_level(&self, avg_transaction_count: u32) -> CongestionLevel {
        // Baseado em observações empíricas da rede Solana
        match avg_transaction_count {
            0..=50 => CongestionLevel::Low,
            51..=150 => CongestionLevel::Medium,
            151..=300 => CongestionLevel::High,
            _ => CongestionLevel::Critical,
        }
    }
    
    /// Estimar tempo de confirmação baseado no congestionamento
    fn estimate_confirmation_time(&self, congestion_level: &CongestionLevel) -> Duration {
        match congestion_level {
            CongestionLevel::Low => Duration::from_secs(5),
            CongestionLevel::Medium => Duration::from_secs(15),
            CongestionLevel::High => Duration::from_secs(30),
            CongestionLevel::Critical => Duration::from_secs(60),
        }
    }
    
    /// NOVO 2024: Criar instrução de compute budget com priority fee dinâmico
    pub async fn create_priority_fee_instruction(&self) -> Result<Instruction, PriorityFeeError> {
        let recommendation = self.get_recommended_fee().await?;
        
        Ok(ComputeBudgetInstruction::set_compute_unit_price(
            recommendation.recommended_fee
        ))
    }
    
    /// NOVO 2024: Criar instruções de compute budget completas para arbitragem
    pub async fn create_compute_budget_instructions(
        &self,
        compute_unit_limit: Option<u32>,
    ) -> Result<Vec<Instruction>, PriorityFeeError> {
        let mut instructions = Vec::new();
        
        // Set compute unit limit
        let cu_limit = compute_unit_limit.unwrap_or(DEFAULT_COMPUTE_UNIT_LIMIT);
        instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(cu_limit));
        
        // Set priority fee
        let priority_fee_ix = self.create_priority_fee_instruction().await?;
        instructions.push(priority_fee_ix);
        
        Ok(instructions)
    }
    
    /// Obter estatísticas de congestionamento
    pub async fn get_congestion_stats(&self) -> CongestionStats {
        let congestion_history = self.congestion_history.read().await;
        
        if congestion_history.is_empty() {
            return CongestionStats::default();
        }
        
        let recent_samples: Vec<_> = congestion_history
            .iter()
            .rev()
            .take(10)
            .collect();
        
        let avg_fee = recent_samples.iter()
            .map(|s| s.average_fee)
            .sum::<u64>() / recent_samples.len() as u64;
        
        let median_fee = recent_samples.iter()
            .map(|s| s.median_fee)
            .sum::<u64>() / recent_samples.len() as u64;
        
        let p95_fee = recent_samples.iter()
            .map(|s| s.p95_fee)
            .sum::<u64>() / recent_samples.len() as u64;
        
        let avg_tx_count = recent_samples.iter()
            .map(|s| s.transaction_count)
            .sum::<u32>() / recent_samples.len() as u32;
        
        CongestionStats {
            sample_count: recent_samples.len(),
            average_fee,
            median_fee,
            p95_fee,
            average_transaction_count: avg_tx_count,
            congestion_level: self.calculate_congestion_level(avg_tx_count),
            data_age: congestion_history.last()
                .map(|s| s.timestamp.elapsed())
                .unwrap_or(Duration::from_secs(0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CongestionStats {
    pub sample_count: usize,
    pub average_fee: u64,
    pub median_fee: u64,
    pub p95_fee: u64,
    pub average_transaction_count: u32,
    pub congestion_level: CongestionLevel,
    pub data_age: Duration,
}

impl Default for CongestionStats {
    fn default() -> Self {
        Self {
            sample_count: 0,
            average_fee: 0,
            median_fee: 0,
            p95_fee: 0,
            average_transaction_count: 0,
            congestion_level: CongestionLevel::Medium,
            data_age: Duration::from_secs(0),
        }
    }
}

/// NOVO 2024: Helper para calcular fee baseado em urgência
pub fn calculate_urgency_based_fee(
    base_fee: u64,
    max_fee: u64,
    urgency_level: f64, // 0.0 - 1.0
) -> u64 {
    if urgency_level <= 0.0 {
        base_fee
    } else if urgency_level >= 1.0 {
        max_fee
    } else {
        let fee_range = max_fee - base_fee;
        let urgency_fee = (fee_range as f64 * urgency_level) as u64;
        base_fee + urgency_fee
    }
}

/// NOVO 2024: Converter SOL para lamports para priority fees
pub fn sol_to_lamports_priority_fee(sol_amount: f64) -> u64 {
    (sol_amount * LAMPORTS_PER_SOL as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_urgency_based_fee_calculation() {
        let base_fee = 1000;
        let max_fee = 10000;
        
        assert_eq!(calculate_urgency_based_fee(base_fee, max_fee, 0.0), base_fee);
        assert_eq!(calculate_urgency_based_fee(base_fee, max_fee, 1.0), max_fee);
        assert_eq!(calculate_urgency_based_fee(base_fee, max_fee, 0.5), 5500);
    }
    
    #[test]
    fn test_sol_to_lamports_conversion() {
        assert_eq!(sol_to_lamports_priority_fee(0.001), 1_000_000);
        assert_eq!(sol_to_lamports_priority_fee(0.05), 50_000_000);
    }
    
    #[test]
    fn test_congestion_level_calculation() {
        let rpc_client = Arc::new(RpcClient::new("http://localhost:8899".to_string()));
        let calculator = DynamicPriorityFeeCalculator::new(rpc_client, 1000, 50000);
        
        assert_eq!(calculator.calculate_congestion_level(25), CongestionLevel::Low);
        assert_eq!(calculator.calculate_congestion_level(100), CongestionLevel::Medium);
        assert_eq!(calculator.calculate_congestion_level(200), CongestionLevel::High);
        assert_eq!(calculator.calculate_congestion_level(400), CongestionLevel::Critical);
    }
} 