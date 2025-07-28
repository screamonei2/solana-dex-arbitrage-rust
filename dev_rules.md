# DEV RULES - Padrões de Desenvolvimento

*Convenções de código, clean code e boas práticas para o Bot de Arbitragem BONK/SOL Solana*

## ⚠️ ATUALIZAÇÕES CRÍTICAS 2024

**FERRAMENTAS OBRIGATÓRIAS:**
- **Rust 1.88+** - Melhores práticas e performance
- **Anchor Framework 0.31+** - Recursos modernos e segurança
- **Solana CLI 1.18+** - Comandos atualizados
- **Jupiter SDK v6** - API endpoints atualizados
- **Professional RPC** - Helius/QuickNode/Alchemy obrigatório

**MEV PROTECTION ATUALIZADA:**
- Jito mempool público **SUSPENSO** (março 2024)
- Priority fees dinâmicas **OBRIGATÓRIAS**
- Rate limiting baseado no provider
- Failover RPC automático

---

## 1. Convenções de Nomenclatura

### 1.1 Rust Naming Conventions

**1.1.a** **Variáveis e Funções** (snake_case)
```rust
// ✅ CORRETO
let bonk_balance = get_token_balance();
let sol_price = calculate_sol_price();
fn execute_arbitrage_strategy() -> Result<()> {}

// ❌ INCORRETO
let bonkBalance = getTokenBalance();
let SolPrice = CalculateSolPrice();
fn ExecuteArbitrageStrategy() -> Result<()> {}
```

**1.1.b** **Structs e Enums** (PascalCase)
```rust
// ✅ CORRETO
struct ArbitrageBot {
    dex_clients: HashMap<String, Box<dyn DexClient>>,
}

enum DexType {
    Raydium,
    Orca,
    Jupiter,
    Meteora,
}

// ❌ INCORRETO
struct arbitrage_bot {}
enum dex_type {}
```

**1.1.c** **Constantes** (SCREAMING_SNAKE_CASE)
```rust
// ✅ CORRETO
const MAX_SLIPPAGE_BPS: u16 = 50;
const BONK_MINT_ADDRESS: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);

// ❌ INCORRETO
const maxSlippageBps: u16 = 50;
const bonk_mint_address: &str = "...";
```

### 1.2 Nomenclatura Específica do Domínio

**1.2.a** **Prefixos por Componente**
```rust
// DEX-specific
struct RaydiumClient {}
struct OrcaClient {}
struct JupiterClient {}

// Strategy-specific
struct DirectArbitrageStrategy {}
struct TriangularArbitrageStrategy {}
struct MultiDexArbitrageStrategy {}

// Monitoring-specific
struct PriceMonitor {}
struct MetricsCollector {}
struct AlertManager {}
```

**1.2.b** **Sufixos por Tipo**
```rust
// Configurações
struct BotConfig {}
struct DexConfig {}
struct SecurityConfig {}

// Resultados
struct ArbitrageResult {}
struct SwapResult {}
struct QuoteResult {}

// Erros
enum ArbitrageError {}
enum DexError {}
enum ValidationError {}
```

## 2. Estrutura de Arquivos e Módulos

### 2.1 Organização de Módulos
**Referência**: [6.2 Estrutura do Projeto](ALL_BASE.md#62-estrutura-do-projeto)

```rust
// src/lib.rs - Ponto de entrada principal
pub mod config;
pub mod dex;
pub mod arbitrage;
pub mod monitoring;
pub mod security;
pub mod utils;

// src/dex/mod.rs - Módulo de DEXs
pub mod raydium;
pub mod orca;
pub mod jupiter;
pub mod meteora;
pub mod traits;

pub use traits::DexClient;
```

**2.1.a** **Regra de Visibilidade**
```rust
// ✅ CORRETO - Exposição mínima necessária
pub struct ArbitrageBot {
    rpc_client: RpcClient,           // privado
    pub config: BotConfig,           // público quando necessário
    dex_clients: HashMap<...>,       // privado
}

impl ArbitrageBot {
    pub fn new(config: BotConfig) -> Self { ... }  // público
    fn validate_config(&self) -> Result<()> { ... } // privado
    pub async fn start(&mut self) -> Result<()> { ... } // público
}
```

### 2.2 Imports e Dependencies

**2.2.a** **Ordem de Imports**
```rust
// 1. Standard library
use std::collections::HashMap;
use std::time::Duration;

// 2. External crates
use anyhow::{Result, Context};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};

// 3. Solana crates
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;

// 4. Local modules
use crate::config::BotConfig;
use crate::dex::{DexClient, DexError};
use crate::arbitrage::ArbitrageStrategy;
```

**2.2.b** **Agrupamento de Imports**
```rust
// ✅ CORRETO - Agrupado por funcionalidade
use crate::dex::{
    raydium::RaydiumClient,
    orca::OrcaClient,
    jupiter::JupiterClient,
    traits::DexClient,
};

// ❌ INCORRETO - Imports separados
use crate::dex::raydium::RaydiumClient;
use crate::dex::orca::OrcaClient;
use crate::dex::jupiter::JupiterClient;
use crate::dex::traits::DexClient;
```

## 3. Padrões de Código

### 3.1 Error Handling

**3.1.a** **Uso de Result e Option**
```rust
// ✅ CORRETO - Propagação de erros
async fn get_quote(
    &self,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
) -> Result<Quote, DexError> {
    let response = self.client
        .get_quote(input_mint, output_mint, amount)
        .await
        .context("Failed to fetch quote from DEX")?;
    
    self.validate_quote(&response)
        .context("Quote validation failed")?;
    
    Ok(response)
}

// ❌ INCORRETO - Panic ou unwrap
fn get_quote_bad(&self) -> Quote {
    let response = self.client.get_quote().unwrap(); // NUNCA!
    response
}
```

**3.1.b** **Custom Error Types**
```rust
// ✅ CORRETO - Erros específicos do domínio
#[derive(Debug, thiserror::Error)]
pub enum ArbitrageError {
    #[error("Insufficient profit: expected {expected}, got {actual}")]
    InsufficientProfit { expected: f64, actual: f64 },
    
    #[error("DEX error: {0}")]
    DexError(#[from] DexError),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}
```

### 3.2 Async Programming

**3.2.a** **Async Best Practices**
```rust
// ✅ CORRETO - Operações paralelas
async fn get_quotes_from_all_dexs(
    &self,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
) -> Result<Vec<Quote>> {
    let futures = self.dex_clients.iter().map(|(name, client)| {
        let input = *input_mint;
        let output = *output_mint;
        async move {
            client.get_quote(&input, &output, amount)
                .await
                .map(|quote| (name.clone(), quote))
        }
    });
    
    let results = futures::future::join_all(futures).await;
    
    // Processar resultados...
    Ok(quotes)
}

// ❌ INCORRETO - Operações sequenciais
async fn get_quotes_sequential(&self) -> Result<Vec<Quote>> {
    let mut quotes = Vec::new();
    for client in &self.dex_clients {
        let quote = client.get_quote().await?; // Sequencial!
        quotes.push(quote);
    }
    Ok(quotes)
}
```

**3.2.b** **Timeout e Cancellation**
```rust
// ✅ CORRETO - Com timeout
use tokio::time::{timeout, Duration};

async fn get_quote_with_timeout(
    &self,
    params: QuoteParams,
) -> Result<Quote, ArbitrageError> {
    let quote_future = self.client.get_quote(params);
    
    match timeout(Duration::from_secs(5), quote_future).await {
        Ok(Ok(quote)) => Ok(quote),
        Ok(Err(e)) => Err(ArbitrageError::DexError(e)),
        Err(_) => Err(ArbitrageError::Timeout),
    }
}
```

### 3.3 Logging e Debugging

**3.3.a** **Structured Logging**
```rust
use tracing::{info, warn, error, debug, instrument};

// ✅ CORRETO - Logs estruturados
#[instrument(skip(self), fields(dex = %dex_name, amount = %amount))]
async fn execute_swap(
    &self,
    dex_name: &str,
    amount: u64,
) -> Result<SwapResult> {
    info!("Starting swap execution");
    
    let quote = self.get_quote(amount).await
        .map_err(|e| {
            error!(error = %e, "Failed to get quote");
            e
        })?;
    
    debug!(quote = ?quote, "Received quote from DEX");
    
    if quote.price_impact > 0.05 {
        warn!(
            price_impact = %quote.price_impact,
            "High price impact detected"
        );
    }
    
    // Executar swap...
    info!("Swap executed successfully");
    Ok(result)
}
```

**3.3.b** **Debug Traits**
```rust
// ✅ CORRETO - Implementar Debug para structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub fees: u64,
    #[serde(skip)] // Não serializar dados sensíveis
    pub internal_data: Option<InternalQuoteData>,
}

// Para dados sensíveis, implementar Debug customizado
impl std::fmt::Debug for SensitiveConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SensitiveConfig")
            .field("rpc_url", &"[REDACTED]")
            .field("private_key", &"[REDACTED]")
            .field("timeout", &self.timeout)
            .finish()
    }
}
```

## 4. Padrões de Configuração

### 4.1 Configuration Management
**Referência**: [9.1 Variáveis de Ambiente](ALL_BASE.md#91-variáveis-de-ambiente-env)

**4.1.a** **Estrutura de Configuração**
```rust
// ✅ CORRETO - Configuração tipada
#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub rpc: RpcConfig,
    pub dexs: DexConfigs,
    pub arbitrage: ArbitrageConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcConfig {
    pub primary_url: String,
    pub backup_urls: Vec<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl BotConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = envy::from_env::<Self>()
            .context("Failed to load configuration from environment")?;
        
        config.validate()
            .context("Configuration validation failed")?;
        
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        if self.rpc.timeout_seconds == 0 {
            return Err(ConfigError::InvalidValue {
                field: "rpc.timeout_seconds".to_string(),
                message: "Timeout must be greater than 0".to_string(),
            });
        }
        
        // Mais validações...
        Ok(())
    }
}
```

### 4.2 Environment Variables (Atualizado 2024)

**4.2.a** **RPC Configuration (Obrigatório)**
```bash
# ✅ CORRETO - Provider profissional obrigatório
ARB_BOT_RPC_PRIMARY_URL="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
ARB_BOT_RPC_BACKUP_URLS="https://api.mainnet-beta.solana.com,https://solana-api.projectserum.com"
ARB_BOT_RPC_TIMEOUT_SECONDS=30
ARB_BOT_RPC_MAX_RETRIES=3
ARB_BOT_RPC_RATE_LIMIT=500  # RPS baseado no plano

# ❌ INCORRETO - Endpoint público em produção
ARB_BOT_RPC_PRIMARY_URL="https://api.mainnet-beta.solana.com"  # NUNCA!
```

**4.2.b** **MEV Protection (Atualizado 2024)**
```bash
# ✅ CORRETO - Configuração pós-Jito
ARB_BOT_JITO_ENABLED=false  # Desabilitado por padrão
ARB_BOT_JITO_MIN_TIP_LAMPORTS=10000  # Mínimo obrigatório
ARB_BOT_PRIORITY_FEE_DYNAMIC=true  # Obrigatório
ARB_BOT_MAX_PRIORITY_FEE_LAMPORTS=50000
ARB_BOT_SLIPPAGE_ADAPTIVE=true

# ❌ INCORRETO - Configuração antiga
ARB_BOT_JITO_ENABLED=true  # Mempool público suspenso!
```

**4.2.c** **Naming Convention**
```bash
# ✅ CORRETO - Prefixo consistente
ARB_BOT_SECURITY_MAX_LOSS_BPS=100
ARB_BOT_MONITORING_METRICS_PORT=9090
ARB_BOT_FAILOVER_THRESHOLD_MS=5000

# ❌ INCORRETO - Inconsistente
TIMEOUT=30
max_loss=100
```

## 5. Padrões de Teste

### 5.1 Unit Tests

**5.1.a** **Estrutura de Testes**
```rust
// ✅ CORRETO - Testes bem estruturados
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_arbitrage_detection_profitable_opportunity() {
        // Arrange
        let mut mock_raydium = MockDexClient::new();
        mock_raydium
            .expect_get_quote()
            .with(eq(BONK_MINT), eq(SOL_MINT), eq(1000000))
            .returning(|_, _, _| Ok(Quote {
                input_amount: 1000000,
                output_amount: 100,
                price_impact: 0.01,
                fees: 2500,
            }));
        
        let mut mock_orca = MockDexClient::new();
        mock_orca
            .expect_get_quote()
            .with(eq(SOL_MINT), eq(BONK_MINT), eq(100))
            .returning(|_, _, _| Ok(Quote {
                input_amount: 100,
                output_amount: 1010000, // Profit!
                price_impact: 0.01,
                fees: 200,
            }));
        
        let strategy = DirectArbitrageStrategy::new(
            Box::new(mock_raydium),
            Box::new(mock_orca),
        );
        
        // Act
        let opportunity = strategy
            .detect_opportunity(BONK_MINT, SOL_MINT, 1000000)
            .await
            .unwrap();
        
        // Assert
        assert!(opportunity.is_profitable());
        assert_eq!(opportunity.profit_bps(), 75); // ~0.75% profit
    }
    
    #[test]
    fn test_config_validation_invalid_timeout() {
        // Arrange
        let config = BotConfig {
            rpc: RpcConfig {
                timeout_seconds: 0, // Invalid!
                ..Default::default()
            },
            ..Default::default()
        };
        
        // Act & Assert
        assert!(config.validate().is_err());
    }
}
```

### 5.2 Integration Tests

**5.2.a** **Testes de Integração**
```rust
// tests/integration_test.rs
use arbitrage_bot::*;
use solana_test_validator::TestValidator;

#[tokio::test]
async fn test_full_arbitrage_flow_on_devnet() {
    // Setup test validator
    let test_validator = TestValidator::with_no_fees(
        solana_sdk::pubkey::Pubkey::new_unique(),
    );
    
    let rpc_url = test_validator.rpc_url();
    
    // Setup bot with test configuration
    let config = BotConfig {
        rpc: RpcConfig {
            primary_url: rpc_url,
            ..Default::default()
        },
        ..test_config()
    };
    
    let mut bot = ArbitrageBot::new(config).await.unwrap();
    
    // Execute test scenario
    let result = bot.run_single_cycle().await;
    
    // Verify results
    assert!(result.is_ok());
}
```

## 6. Padrões de Documentação

### 6.1 Code Documentation

**6.1.a** **Rustdoc Comments**
```rust
/// Executes a direct arbitrage strategy between two DEXs.
/// 
/// This function identifies price differences between two DEXs for the same
/// token pair and executes trades to capture the profit.
/// 
/// # Arguments
/// 
/// * `input_mint` - The mint address of the input token
/// * `output_mint` - The mint address of the output token  
/// * `amount` - The amount of input tokens to trade
/// 
/// # Returns
/// 
/// Returns `Ok(ArbitrageResult)` if the arbitrage was successful,
/// or `Err(ArbitrageError)` if an error occurred.
/// 
/// # Examples
/// 
/// ```rust
/// use solana_sdk::pubkey::Pubkey;
/// 
/// let bonk_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263")?;
/// let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
/// 
/// let result = strategy.execute_arbitrage(
///     &bonk_mint,
///     &sol_mint, 
///     1_000_000
/// ).await?;
/// ```
/// 
/// # Errors
/// 
/// This function will return an error if:
/// * The DEX APIs are unavailable
/// * Insufficient liquidity exists
/// * The arbitrage opportunity is no longer profitable
/// * Network connectivity issues occur
pub async fn execute_arbitrage(
    &self,
    input_mint: &Pubkey,
    output_mint: &Pubkey,
    amount: u64,
) -> Result<ArbitrageResult, ArbitrageError> {
    // Implementation...
}
```

### 6.2 README Templates

**6.2.a** **Module README Template**
```markdown
# Module: DEX Clients

This module contains implementations for various DEX clients used in the arbitrage bot.

## Overview

Each DEX client implements the `DexClient` trait, providing a unified interface for:
- Getting price quotes
- Executing swaps
- Retrieving fee information

## Supported DEXs

- **Raydium**: AMM + CLOB hybrid ([docs](./raydium.rs))
- **Orca**: Concentrated liquidity AMM ([docs](./orca.rs))
- **Jupiter**: DEX aggregator ([docs](./jupiter.rs))
- **Meteora**: Dynamic liquidity AMM ([docs](./meteora.rs))

## Usage

```rust
use crate::dex::{DexClient, raydium::RaydiumClient};

let client = RaydiumClient::new(config).await?;
let quote = client.get_quote(&input_mint, &output_mint, amount).await?;
```

## Configuration

See [config.rs](../config.rs) for configuration options.
```

## 7. Performance Guidelines

### 7.1 Memory Management

**7.1.a** **Avoid Unnecessary Allocations**
```rust
// ✅ CORRETO - Reutilizar buffers
struct QuoteCache {
    buffer: Vec<Quote>,
}

impl QuoteCache {
    fn update_quotes(&mut self, new_quotes: &[Quote]) {
        self.buffer.clear();
        self.buffer.extend_from_slice(new_quotes);
    }
}

// ❌ INCORRETO - Alocações desnecessárias
fn process_quotes(quotes: &[Quote]) -> Vec<String> {
    quotes.iter()
        .map(|q| format!("Quote: {}", q.price)) // Nova String a cada iteração
        .collect()
}
```

**7.1.b** **Use References When Possible**
```rust
// ✅ CORRETO - Usar referências
fn calculate_profit(quote_a: &Quote, quote_b: &Quote) -> f64 {
    // Cálculo sem ownership
}

// ❌ INCORRETO - Ownership desnecessário
fn calculate_profit_bad(quote_a: Quote, quote_b: Quote) -> f64 {
    // Consome os valores
}
```

### 7.2 Async Performance

**7.2.a** **Batch Operations**
```rust
// ✅ CORRETO - Operações em lote
async fn update_all_prices(&mut self) -> Result<()> {
    let futures: Vec<_> = self.dex_clients
        .iter()
        .map(|(name, client)| {
            let name = name.clone();
            async move {
                client.get_current_price().await
                    .map(|price| (name, price))
            }
        })
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    for result in results {
        match result {
            Ok((name, price)) => self.prices.insert(name, price),
            Err(e) => warn!("Failed to update price: {}", e),
        };
    }
    
    Ok(())
}
```

## 8. Padrões Modernos Anchor 0.31+ (2024)

### 8.1 Anchor Framework Atualizado
**Referência**: [8.6 Desenvolvimento Moderno com Solana](ALL_BASE.md#86-desenvolvimento-moderno-com-solana-2024)

**8.1.a** **Estrutura de Projeto Moderna**
```rust
// ✅ CORRETO - Anchor 0.31+ com features modernas
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[program]
pub mod arbitrage_bot {
    use super::*;
    
    #[derive(Accounts)]
    pub struct ExecuteArbitrage<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        
        #[account(
            mut,
            constraint = user_token_account.owner == user.key(),
            constraint = user_token_account.mint == token_mint.key()
        )]
        pub user_token_account: Account<'info, TokenAccount>,
        
        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
    }
}
```

**8.1.b** **Error Handling Moderno**
```rust
// ✅ CORRETO - Custom errors com Anchor 0.31+
#[error_code]
pub enum ArbitrageError {
    #[msg("Insufficient profit for arbitrage")]
    InsufficientProfit,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Invalid DEX configuration")]
    InvalidDexConfig,
}

// Uso em instruções
pub fn execute_arbitrage(ctx: Context<ExecuteArbitrage>) -> Result<()> {
    require!(profit > min_profit, ArbitrageError::InsufficientProfit);
    Ok(())
}
```

**8.1.c** **Testing com Anchor**
```rust
// ✅ CORRETO - Testes modernos
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    use solana_program_test::*;
    
    #[tokio::test]
    async fn test_arbitrage_execution() {
        let program_test = ProgramTest::new(
            "arbitrage_bot",
            arbitrage_bot::id(),
            processor!(arbitrage_bot::entry),
        );
        
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        
        // Test implementation...
    }
}
```

## 9. Security Guidelines

### 9.1 Input Validation
**Referência**: [8.1 Padrões de Segurança](ALL_BASE.md#81-padrões-de-segurança)

```rust
// ✅ CORRETO - Validação rigorosa
fn validate_trade_amount(amount: u64) -> Result<u64, ValidationError> {
    const MIN_AMOUNT: u64 = 1000; // 0.001 tokens
    const MAX_AMOUNT: u64 = 1_000_000_000; // 1000 tokens
    
    if amount < MIN_AMOUNT {
        return Err(ValidationError::AmountTooSmall { 
            amount, 
            minimum: MIN_AMOUNT 
        });
    }
    
    if amount > MAX_AMOUNT {
        return Err(ValidationError::AmountTooLarge { 
            amount, 
            maximum: MAX_AMOUNT 
        });
    }
    
    Ok(amount)
}

fn validate_pubkey(key_str: &str) -> Result<Pubkey, ValidationError> {
    Pubkey::from_str(key_str)
        .map_err(|_| ValidationError::InvalidPubkey { 
            key: key_str.to_string() 
        })
}
```

### 9.2 Safe Math Operations (Atualizado 2024)

**9.2.a** **Matemática Segura com Priority Fees**
```rust
// ✅ CORRETO - Incluindo priority fees dinâmicas
fn calculate_profit_with_priority_fees(
    input_amount: u64,
    output_amount: u64,
    swap_fees: u64,
    priority_fee: u64,  // Nova para 2024
    network_fee: u64,
) -> Result<i64, ArithmeticError> {
    let total_fees = swap_fees
        .checked_add(priority_fee)
        .and_then(|f| f.checked_add(network_fee))
        .ok_or(ArithmeticError::Overflow)?;
    
    let total_cost = input_amount
        .checked_add(total_fees)
        .ok_or(ArithmeticError::Overflow)?;
    
    let profit = (output_amount as i64)
        .checked_sub(total_cost as i64)
        .ok_or(ArithmeticError::Overflow)?;
    
    Ok(profit)
}

// ✅ CORRETO - Validação de slippage adaptativo
fn calculate_adaptive_slippage(
    base_slippage_bps: u16,
    volatility_factor: f64,
    max_slippage_bps: u16,
) -> Result<u16, ArithmeticError> {
    let adjusted_slippage = (base_slippage_bps as f64 * volatility_factor) as u16;
    
    Ok(adjusted_slippage.min(max_slippage_bps))
}

// ❌ INCORRETO - Não considera priority fees
fn calculate_profit_unsafe(input: u64, output: u64, fees: u64) -> i64 {
    (output - input - fees) as i64 // Panic em overflow!
}
```

**9.2.b** **Rate Limiting Calculations**
```rust
// ✅ CORRETO - Cálculo de rate limiting
fn calculate_request_delay(
    requests_per_second: u32,
    current_requests: u32,
) -> Result<Duration, ArithmeticError> {
    if current_requests >= requests_per_second {
        let delay_ms = 1000u64
            .checked_div(requests_per_second as u64)
            .ok_or(ArithmeticError::DivisionByZero)?;
        
        Ok(Duration::from_millis(delay_ms))
    } else {
        Ok(Duration::from_millis(0))
    }
}
```

---

## Checklist de Code Review (Atualizado 2024)

### ⚠️ Verificações Críticas 2024
- [ ] **RPC Provider Profissional** - Nunca usar endpoints públicos em produção
- [ ] **MEV Protection Atualizada** - Priority fees dinâmicas implementadas
- [ ] **Jito Configuration** - JITO_ENABLED=false por padrão
- [ ] **Rate Limiting** - Controle baseado no plano do provider
- [ ] **Failover RPC** - Sistema de backup configurado
- [ ] **Rust 1.88+** - Toolchain atualizado
- [ ] **Anchor 0.31+** - Framework moderno utilizado

### Funcionalidade
- [ ] Código atende aos requisitos
- [ ] Lógica está correta
- [ ] Edge cases são tratados
- [ ] Testes adequados incluídos
- [ ] Priority fees calculadas corretamente
- [ ] Slippage adaptativo implementado

### Qualidade
- [ ] Nomenclatura clara e consistente
- [ ] Funções têm responsabilidade única
- [ ] Código é legível e bem documentado
- [ ] Sem duplicação desnecessária
- [ ] Anchor patterns seguidos (se aplicável)

### Performance
- [ ] Sem alocações desnecessárias
- [ ] Operações assíncronas otimizadas
- [ ] Complexidade algorítmica adequada
- [ ] Recursos são liberados adequadamente
- [ ] Rate limiting respeitado
- [ ] Connection pooling implementado

### Segurança
- [ ] Input validation implementada
- [ ] Operações matemáticas seguras (com priority fees)
- [ ] Dados sensíveis protegidos
- [ ] Error handling adequado
- [ ] Circuit breakers configurados
- [ ] API keys não expostas

### Manutenibilidade
- [ ] Código segue padrões estabelecidos
- [ ] Dependências são justificadas e atualizadas
- [ ] Configuração é externalizável
- [ ] Logs e métricas adequados
- [ ] Documentação atualizada
- [ ] Runbooks criados para produção

### Compliance 2024
- [ ] Variáveis de ambiente seguem padrão ARB_BOT_*
- [ ] Configurações de produção validadas
- [ ] Monitoring e alertas configurados
- [ ] Backup e recovery procedures documentados

---

*Estes padrões devem ser seguidos consistentemente em todo o projeto. Use este documento como referência durante desenvolvimento e code reviews.*