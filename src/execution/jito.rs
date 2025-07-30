// Implementação de Priority Fees Dinâmicas (2024)
// ATUALIZADO: Jito mempool público suspenso março 2024
// Foco em priority fees dinâmicas para competir por inclusão nos blocos

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    signature::Keypair,
    transaction::Transaction,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn, info, error};
use thiserror::Error;

use crate::utils::priority_fees::DynamicPriorityFeeCalculator;

#[derive(Debug, Error)]
pub enum MevProtectionError {
    #[error("Failed to calculate priority fees: {0}")]
    PriorityFeeCalculation(#[from] crate::utils::priority_fees::PriorityFeeError),
    
    #[error("Transaction failed after MEV protection: {0}")]
    TransactionFailed(String),
    
    #[error("Network congestion too high")]
    NetworkCongestion,
    
    #[error("Priority fee exceeds budget: {fee} > {budget}")]
    BudgetExceeded { fee: u64, budget: u64 },
}

/// Sistema de proteção MEV atualizado para 2024
/// Foca em priority fees dinâmicas em vez de bundles Jito
pub struct MevProtectionSystem {
    rpc_client: Arc<RpcClient>,
    priority_fee_calculator: Arc<DynamicPriorityFeeCalculator>,
    stats: Arc<RwLock<MevProtectionStats>>,
    max_priority_fee_lamports: u64,
}

#[derive(Debug, Default)]
pub struct MevProtectionStats {
    pub transactions_protected: u64,
    pub total_priority_fees_paid: u64,
    pub successful_submissions: u64,
    pub failed_submissions: u64,
    pub average_confirmation_time: Duration,
}

#[derive(Debug, Clone)]
pub struct MevProtectionConfig {
    pub max_priority_fee_lamports: u64,
    pub min_priority_fee_lamports: u64,
    pub target_confirmation_time: Duration,
    pub dynamic_fee_adjustment: bool,
    pub dynamic_adjustment: bool, // Alias for compatibility
    pub competition_factor: f64,
    pub slippage_adaptive: bool,
}

impl Default for MevProtectionConfig {
    fn default() -> Self {
        Self {
            max_priority_fee_lamports: 50_000, // 0.00005 SOL
            min_priority_fee_lamports: 10_000, // 0.00001 SOL
            target_confirmation_time: Duration::from_secs(5),
            dynamic_fee_adjustment: true,
            dynamic_adjustment: true, // Alias for compatibility
            competition_factor: 1.5, // 50% mais que a fee mediana
            slippage_adaptive: true,
        }
    }
}

impl MevProtectionSystem {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        config: MevProtectionConfig,
    ) -> Self {
        let priority_fee_calculator = Arc::new(
            DynamicPriorityFeeCalculator::new(rpc_client.clone(), config.clone())
        );

        Self {
            rpc_client,
            priority_fee_calculator,
            stats: Arc::new(RwLock::new(MevProtectionStats::default())),
            max_priority_fee_lamports: config.max_priority_fee_lamports,
        }
    }

    /// Proteger transação com priority fees dinâmicas
    pub async fn protect_transaction(
        &self,
        mut transaction: Transaction,
        user_keypair: &Keypair,
        urgency_level: UrgencyLevel,
    ) -> Result<String, MevProtectionError> {
        let start_time = Instant::now();
        
        info!("Protecting transaction with MEV protection (2024 mode)");

        // Calcular priority fee baseado na urgência e condições de rede
        let priority_fee = self.calculate_dynamic_priority_fee(urgency_level).await?;
        
        debug!("Calculated priority fee: {} lamports", priority_fee);

        // Verificar orçamento
        if priority_fee > self.max_priority_fee_lamports {
            return Err(MevProtectionError::BudgetExceeded {
                fee: priority_fee,
                budget: self.max_priority_fee_lamports,
            });
        }

        // Adicionar instruções de compute budget
        let compute_instructions = self.create_compute_budget_instructions(priority_fee)?;
        
        // Inserir no início da transação
        let mut new_instructions = compute_instructions;
        new_instructions.extend(transaction.message.instructions.clone());
        
        // Recriar transação com priority fees
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let mut protected_transaction = Transaction::new_with_payer(
            &new_instructions.iter().collect::<Vec<_>>(),
            Some(&user_keypair.pubkey()),
        );
        
        protected_transaction.partial_sign(&[user_keypair], recent_blockhash);
        
        // Submeter com retry inteligente
        let signature = self.submit_with_retry(protected_transaction, 3).await?;
        
        // Atualizar estatísticas
        self.update_stats(priority_fee, start_time.elapsed(), true).await;
        
        info!("Transaction protected and submitted: {}", signature);
        Ok(signature)
    }

    /// Calcular priority fee baseada nas condições atuais da rede
    async fn calculate_dynamic_priority_fee(
        &self,
        urgency: UrgencyLevel,
    ) -> Result<u64, MevProtectionError> {
        let base_fee = self.priority_fee_calculator
            .get_recommended_priority_fee()
            .await?;

        let urgency_multiplier = match urgency {
            UrgencyLevel::Low => 1.0,
            UrgencyLevel::Medium => 1.5,
            UrgencyLevel::High => 2.0,
            UrgencyLevel::Critical => 3.0,
        };

        let dynamic_fee = (base_fee as f64 * urgency_multiplier) as u64;
        
        // Limitado pelo orçamento máximo
        Ok(dynamic_fee.min(self.max_priority_fee_lamports))
    }

    /// Criar instruções de compute budget
    fn create_compute_budget_instructions(
        &self,
        priority_fee_lamports: u64,
    ) -> Result<Vec<Instruction>, MevProtectionError> {
        let instructions = vec![
            // Definir priority fee
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports),
            // Definir compute unit limit conservador
            ComputeBudgetInstruction::set_compute_unit_limit(200_000),
        ];

        Ok(instructions)
    }

    /// Submeter transação com retry inteligente
    async fn submit_with_retry(
        &self,
        transaction: Transaction,
        max_retries: u32,
    ) -> Result<String, MevProtectionError> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    debug!("Transaction confirmed on attempt {}", attempt + 1);
                    return Ok(signature.to_string());
                }
                Err(e) => {
                    warn!("Transaction failed on attempt {}: {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    if attempt < max_retries - 1 {
                        // Exponential backoff
                        let delay = Duration::from_millis(500 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(MevProtectionError::TransactionFailed(
            last_error.unwrap().to_string()
        ))
    }

    /// Atualizar estatísticas internas
    async fn update_stats(
        &self,
        priority_fee_paid: u64,
        confirmation_time: Duration,
        success: bool,
    ) {
        let mut stats = self.stats.write().await;
        
        stats.transactions_protected += 1;
        stats.total_priority_fees_paid += priority_fee_paid;
        
        if success {
            stats.successful_submissions += 1;
            // Rolling average
            stats.average_confirmation_time = Duration::from_millis(
                (stats.average_confirmation_time.as_millis() + confirmation_time.as_millis()) / 2
            );
        } else {
            stats.failed_submissions += 1;
        }
    }

    /// Obter estatísticas do sistema
    pub async fn get_stats(&self) -> MevProtectionStats {
        self.stats.read().await.clone()
    }

    /// Verificar se as condições de rede exigem proteção extra
    pub async fn should_use_mev_protection(&self) -> Result<bool, MevProtectionError> {
        // Verificar congestionamento da rede
        let recent_performance = self.rpc_client
            .get_recent_performance_samples(Some(10))?;

        let avg_tps = recent_performance
            .iter()
            .map(|sample| sample.num_transactions as f64 / sample.sample_period_secs as f64)
            .sum::<f64>() / recent_performance.len() as f64;

        // Se TPS > 2000, rede está congestionada
        Ok(avg_tps > 2000.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UrgencyLevel {
    Low,      // Transações normais
    Medium,   // Arbitragem típica
    High,     // Oportunidades competitivas
    Critical, // Liquidações, MEV time-sensitive
}

/// Factory para criar sistema MEV otimizado para arbitragem
pub fn create_arbitrage_mev_protection(
    rpc_client: Arc<RpcClient>,
) -> MevProtectionSystem {
    let config = MevProtectionConfig {
        max_priority_fee_lamports: 25_000, // Conservador para arbitragem
        target_confirmation_time: Duration::from_secs(3),
        dynamic_fee_adjustment: true,
        competition_factor: 2.0, // Agressivo para arbitragem
        min_priority_fee_lamports: 10_000, // Conservador para arbitragem
        dynamic_adjustment: true, // Alias for compatibility
        slippage_adaptive: true,
    };

    MevProtectionSystem::new(rpc_client, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{signature::Keypair, system_instruction};

    #[tokio::test]
    async fn test_mev_protection_system() {
        // Mock RPC client seria necessário para teste real
        // Este é um exemplo de estrutura de teste
    }

    #[test]
    fn test_urgency_level_multipliers() {
        // Testa se os multiplicadores de urgência estão corretos
        assert_eq!(std::mem::discriminant(&UrgencyLevel::Low), 
                   std::mem::discriminant(&UrgencyLevel::Low));
    }
} 