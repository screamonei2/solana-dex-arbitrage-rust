# BASE.md - Manual Completo para Bot de Arbitragem Solana

## Índice
1. [Visão Geral do Ecossistema Solana](#visão-geral-do-ecossistema-solana)
2. [DEXs Principais que Suportam BONK/SOL](#dexs-principais-que-suportam-bonksol)
3. [APIs e SDKs](#apis-e-sdks)
4. [Arquitetura de Bot de Arbitragem](#arquitetura-de-bot-de-arbitragem)
5. [Cálculos Matemáticos](#cálculos-matemáticos)
6. [Monitoramento em Tempo Real](#monitoramento-em-tempo-real)
7. [Estratégias de Arbitragem](#estratégias-de-arbitragem)
8. [Implementação em Rust](#implementação-em-rust)
9. [Segurança e Boas Práticas](#segurança-e-boas-práticas)
10. [Configuração e Deploy](#configuração-e-deploy)

---

## Visão Geral do Ecossistema Solana

### Características da Rede Solana
- **Throughput**: Até 65.000 transações por segundo
- **Latência**: ~400ms para confirmação
- **Taxas**: Frações de centavo por transação
- **Arquitetura**: Proof of History (PoH) + Proof of Stake (PoS)
- **Linguagens**: Rust, C para smart contracts

### Vantagens para Arbitragem
- Velocidade de execução permite capturar oportunidades pequenas
- Baixas taxas tornam micro-arbitragem viável
- Composabilidade permite transações atômicas complexas
- Ecossistema DeFi maduro com alta liquidez

---

## DEXs Principais que Suportam BONK/SOL

### 1. Raydium
**Tipo**: AMM + CLOB Híbrido
**Endereço do Programa**: `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`

#### Características:
- Integração com Serum order book
- Pools de liquidez concentrada (CLMM)
- Yield farming e staking
- API REST para cotações e swaps

#### SDK e API:
```typescript
// Raydium SDK v2
import { API_URLS } from '@raydium-io/raydium-sdk-v2'

// Endpoint para cotação
const quoteUrl = `${API_URLS.SWAP_HOST}/compute/swap-base-in`
// Parâmetros: inputMint, outputMint, amount, slippageBps, txVersion

// Endpoint para transação
const swapUrl = `${API_URLS.SWAP_HOST}/transaction/swap-base-in`
```

#### Taxas:
- Taxa de swap: 0.25%
- Taxa de protocolo: Variável
- Priority fee: Configurável (recomendado: 30.000 microlamports)

### 2. Orca
**Tipo**: AMM com Liquidez Concentrada (Whirlpool)
**Endereço do Programa**: `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`

#### Características:
- Whirlpools (liquidez concentrada)
- Baixo slippage
- Interface amigável
- Suporte a múltiplas fee tiers

#### SDK:
```typescript
// Orca SDK
import { WhirlpoolContext, buildWhirlpoolClient } from "@orca-so/whirlpools-sdk"

// Para cotações
const quote = await swapQuoteByInputToken(
  whirlpool,
  inputTokenMint,
  tokenAmount,
  slippageTolerance,
  programId,
  fetcher
)
```

#### Taxas:
- Fee tiers: 0.01%, 0.05%, 0.3%, 1%
- Dependem do par e volatilidade

### 3. Jupiter
**Tipo**: Agregador de DEXs
**Endereço do Programa**: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`

#### Características:
- Agrega liquidez de múltiplas DEXs
- Smart routing para melhor preço
- Trade splitting
- Suporte a 20+ DEXs

#### API v6:
```typescript
// Jupiter API v6
const quoteUrl = 'https://quote-api.jup.ag/v6/quote'
const swapUrl = 'https://quote-api.jup.ag/v6/swap'

// Parâmetros para cotação
const params = {
  inputMint: 'So11111111111111111111111111111111111111112', // SOL
  outputMint: 'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263', // BONK
  amount: '1000000000', // 1 SOL em lamports
  slippageBps: '50' // 0.5%
}
```

### 4. Meteora
**Tipo**: AMM Multi-pool
**Endereço do Programa**: `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB`

#### Características:
- Dynamic pools (DYN)
- Pools estáveis para stablecoins
- Yield farming
- Baixas taxas

#### API (via Bitquery):
```graphql
query {
  solana {
    dexTrades(
      baseCurrency: {is: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"}
      quoteCurrency: {is: "So11111111111111111111111111111111111111112"}
      exchangeAddress: {is: "Meteora_Program_Address"}
    ) {
      trades
      quotePrice
      baseAmount
    }
  }
}
```

### 5. OpenBook (Serum v3 Fork)
**Tipo**: Central Limit Order Book (CLOB)
**Endereço do Programa**: `srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX`

#### Características:
- Order book tradicional
- Liquidez profunda
- Integração com outros protocolos
- Suporte completo a BONK

### 6. Phoenix
**Tipo**: CLOB Otimizado
**Endereço do Programa**: `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY`

#### SDKs Disponíveis:
- Rust SDK
- TypeScript SDK
- Python SDK

### 7. Lifinity
**Tipo**: Proactive Market Maker
**Endereço do Programa**: `EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S`

#### Características:
- Oráculo para precificação
- Redução de impermanent loss
- Eficiência de capital

### 8. Saber
**Tipo**: AMM para Stablecoins
**Endereço do Programa**: `SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ`

#### Características:
- Otimizada para stablecoins e wrapped tokens
- Baixo slippage para ativos pareados
- Migrou para nova infraestrutura
- Foco em liquidez e baixas taxas

### 9. Mercurial Finance
**Tipo**: AMM para Stable Swaps
**Endereço do Programa**: `MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky`

#### Características:
- Stable swaps otimizados
- Vaults para yield farming
- SDK TypeScript e Rust disponíveis
- Pools para diversos pares de stablecoins

#### Pools Principais:
- USDC-USDT-PAI
- pSOL-SOL
- stSOL-SOL
- mSOL-SOL

---

## APIs e SDKs

### Bitquery API (Universal)
**Endpoint**: `https://graphql.bitquery.io/`

```graphql
# Monitoramento de trades em tempo real
subscription {
  solana {
    dexTrades(
      baseCurrency: {is: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"}
      quoteCurrency: {is: "So11111111111111111111111111111111111111112"}
    ) {
      transaction {
        signature
      }
      exchange {
        address
        name
      }
      baseAmount
      quoteAmount
      quotePrice
      block {
        timestamp {
          time
        }
      }
    }
  }
}
```

### SolanaTracker API
**Endpoint**: `https://api.solanatracker.io/`
**Taxa**: 0.5% por swap

```typescript
// Swap genérico
const swapData = {
  from: 'So11111111111111111111111111111111111111112',
  to: 'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263',
  amount: 1000000000,
  slippage: 50, // 0.5%
  priorityFee: 0.0001,
  txVersion: 'v0'
}
```

### OKX DEX API
**Endpoint**: `https://www.okx.com/api/v5/dex/`

#### Funcionalidades:
- Simulação de transações
- Broadcast de transações
- Estimativa de compute units
- Proteção MEV

---

## Arquitetura de Bot de Arbitragem

### Componentes Principais

```rust
// Estrutura principal do bot
pub struct ArbitrageBot {
    pub config: BotConfig,
    pub price_monitor: PriceMonitor,
    pub strategy_engine: StrategyEngine,
    pub execution_engine: ExecutionEngine,
    pub risk_manager: RiskManager,
}

pub struct BotConfig {
    pub min_profit_threshold: f64,    // 0.5%
    pub max_slippage: f64,           // 1%
    pub max_position_size: u64,
    pub priority_fee: u64,           // microlamports
    pub compute_unit_limit: u32,
    pub rpc_endpoints: Vec<String>,
    pub dex_priorities: Vec<DexType>,
}
```

### Fluxo de Execução

1. **Monitoramento de Preços**
   - WebSocket connections para cada DEX
   - Yellowstone gRPC para dados em tempo real
   - Cache de preços com timestamps

2. **Detecção de Oportunidades**
   - Comparação de preços entre DEXs
   - Cálculo de profit potencial
   - Validação de liquidez disponível

3. **Execução de Estratégia**
   - Construção de transação atômica
   - Simulação antes da execução
   - Submissão via MEV-aware RPC

4. **Gerenciamento de Risco**
   - Verificação de slippage
   - Monitoramento de posições
   - Circuit breakers

---

## Cálculos Matemáticos

### Fórmula do Produto Constante (AMM)
```
x * y = k

Onde:
- x = quantidade do token A no pool
- y = quantidade do token B no pool  
- k = constante que deve ser mantida
```

### Cálculo de Slippage
```rust
// Slippage em AMM
fn calculate_slippage(pool_x: u64, pool_y: u64, amount_in: u64) -> f64 {
    let k = pool_x * pool_y;
    let new_pool_x = pool_x + amount_in;
    let new_pool_y = k / new_pool_x;
    let amount_out = pool_y - new_pool_y;
    
    let expected_price = pool_y as f64 / pool_x as f64;
    let actual_price = amount_out as f64 / amount_in as f64;
    
    (expected_price - actual_price) / expected_price
}
```

### Cálculo de Arbitragem
```rust
fn calculate_arbitrage_profit(
    price_dex_a: f64,
    price_dex_b: f64,
    amount: u64,
    fees_a: f64,
    fees_b: f64,
    slippage_a: f64,
    slippage_b: f64
) -> f64 {
    let amount_after_fees_a = amount as f64 * (1.0 - fees_a);
    let amount_after_slippage_a = amount_after_fees_a * (1.0 - slippage_a);
    
    let intermediate_amount = amount_after_slippage_a / price_dex_a;
    
    let amount_after_fees_b = intermediate_amount * (1.0 - fees_b);
    let final_amount = amount_after_fees_b * (1.0 - slippage_b) * price_dex_b;
    
    (final_amount - amount as f64) / amount as f64
}
```

---

## Monitoramento em Tempo Real

### Yellowstone gRPC
**Vantagens sobre WebSocket**:
- Maior controle e confiabilidade
- Dados estruturados
- Filtros personalizados
- Suporte a múltiplos fluxos
- Baixa latência e alta estabilidade

```rust
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

// Configuração do cliente
let mut client = GeyserGrpcClient::connect(
    "https://api.mainnet-beta.solana.com",
    None,
    None,
).await?;

// Subscription para accounts
let mut accounts = HashMap::new();
accounts.insert(
    "raydium_pools".to_string(),
    SubscribeRequestFilterAccounts {
        account: vec![],
        owner: vec!["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()],
        filters: vec![],
    },
);

// Subscription para transactions
let mut transactions = HashMap::new();
transactions.insert(
    "dex_swaps".to_string(),
    SubscribeRequestFilterTransactions {
        vote: Some(false),
        failed: Some(false),
        signature: None,
        account_include: vec![
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(), // Raydium
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(), // Orca
        ],
        account_exclude: vec![],
        account_required: vec![],
    },
);
```

### WebSocket Connections
```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Conexão WebSocket para Raydium
let raydium_ws = "wss://api.raydium.io/v2/ws";
let (ws_stream, _) = connect_async(raydium_ws).await?;

// Subscription message
let subscribe_msg = json!({
    "method": "subscribe",
    "params": {
        "channel": "price_updates",
        "market": "BONK/SOL"
    }
});
```

---

## Estratégias de Arbitragem

### 1. Arbitragem Direta (Two-Hop)
```rust
// SOL -> BONK (DEX A) -> SOL (DEX B)
struct DirectArbitrage {
    dex_a: DexType,
    dex_b: DexType,
    token_pair: TokenPair,
    min_profit: f64,
}

impl DirectArbitrage {
    fn execute(&self, amount: u64) -> Result<Transaction> {
        let mut instructions = Vec::new();
        
        // Swap 1: SOL -> BONK on DEX A
        instructions.push(self.build_swap_instruction(
            self.dex_a,
            SOL_MINT,
            BONK_MINT,
            amount,
        )?);
        
        // Swap 2: BONK -> SOL on DEX B
        instructions.push(self.build_swap_instruction(
            self.dex_b,
            BONK_MINT,
            SOL_MINT,
            0, // Use all BONK from previous swap
        )?);
        
        // Profit check instruction
        instructions.push(self.build_profit_check_instruction(amount)?);
        
        Ok(Transaction::new_with_payer(&instructions, Some(&self.payer)))
    }
}
```

### 2. Arbitragem Triangular
```rust
// SOL -> BONK -> USDC -> SOL
struct TriangularArbitrage {
    path: Vec<(DexType, TokenMint, TokenMint)>,
    min_profit: f64,
}

impl TriangularArbitrage {
    fn calculate_opportunity(&self, amount: u64) -> Option<ArbitrageOpportunity> {
        let mut current_amount = amount as f64;
        
        for (dex, token_in, token_out) in &self.path {
            let price = self.get_price(*dex, *token_in, *token_out)?;
            let fee = self.get_fee(*dex, *token_in, *token_out);
            let slippage = self.estimate_slippage(*dex, current_amount);
            
            current_amount = current_amount * price * (1.0 - fee) * (1.0 - slippage);
        }
        
        let profit = (current_amount - amount as f64) / amount as f64;
        
        if profit > self.min_profit {
            Some(ArbitrageOpportunity {
                profit,
                path: self.path.clone(),
                amount,
            })
        } else {
            None
        }
    }
}
```

### 3. Multi-DEX Arbitragem
```rust
// Utiliza Jupiter para encontrar a melhor rota
struct MultiDexArbitrage {
    jupiter_client: JupiterClient,
    monitored_tokens: Vec<TokenMint>,
}

impl MultiDexArbitrage {
    async fn find_opportunities(&self) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();
        
        for token in &self.monitored_tokens {
            // Get best route from Jupiter
            let route = self.jupiter_client.get_route(
                SOL_MINT,
                *token,
                1_000_000_000, // 1 SOL
            ).await?;
            
            // Get reverse route
            let reverse_route = self.jupiter_client.get_route(
                *token,
                SOL_MINT,
                route.out_amount,
            ).await?;
            
            let profit = (reverse_route.out_amount as f64 - 1_000_000_000.0) / 1_000_000_000.0;
            
            if profit > self.min_profit {
                opportunities.push(ArbitrageOpportunity {
                    profit,
                    forward_route: route,
                    reverse_route,
                });
            }
        }
        
        opportunities
    }
}
```

---

## MEV e Jito Integration

### Jito MEV Bot
```rust
// Bot MEV com flashloans atômicos
struct JitoMevBot {
    jito_client: JitoClient,
    yellowstone_client: YellowstoneClient,
    supported_dexs: Vec<DexType>,
}

impl JitoMevBot {
    async fn monitor_mempool(&self) -> Result<()> {
        // Monitor mempool via Yellowstone gRPC
        let mut stream = self.yellowstone_client.subscribe_transactions().await?;
        
        while let Some(transaction) = stream.next().await {
            // Analyze transaction for MEV opportunities
            if let Some(opportunity) = self.analyze_transaction(&transaction).await? {
                // Execute MEV strategy
                self.execute_mev_strategy(opportunity).await?;
            }
        }
        
        Ok(())
    }
    
    async fn execute_mev_strategy(&self, opportunity: MevOpportunity) -> Result<()> {
        // Build bundle with MEV transaction
        let bundle = self.build_jito_bundle(opportunity).await?;
        
        // Submit bundle to Jito
        let bundle_id = self.jito_client.send_bundle(bundle).await?;
        
        // Monitor bundle status
        self.monitor_bundle_status(bundle_id).await?;
        
        Ok(())
    }
}
```

---

## Implementação em Rust

### Dependências Principais
```toml
[dependencies]
# Solana
solana-client = "1.17"
solana-sdk = "1.17"
spl-token = "4.0"
spl-associated-token-account = "2.0"

# DEX SDKs
raydium-sdk = "0.1"
orca-whirlpools-sdk = "0.1"
jupiter-sdk = "0.1"

# Async runtime
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

# Networking
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.20"

# Data streaming
yellowstone-grpc-client = "1.13"
yellowstone-grpc-proto = "1.13"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Math
num-traits = "0.2"
num-bigint = "0.4"

# Logging
log = "0.4"
env_logger = "0.10"

# Configuration
dotenv = "0.15"
config = "0.13"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Metrics
prometheus = "0.13"

# MEV
jito-client = "0.1"

# Base58 encoding
bs58 = "0.4"
```

### Estrutura do Projeto
```
src/
├── main.rs
├── config/
│   ├── mod.rs
│   └── settings.rs
├── dex/
│   ├── mod.rs
│   ├── raydium.rs
│   ├── orca.rs
│   ├── jupiter.rs
│   ├── meteora.rs
│   ├── phoenix.rs
│   ├── saber.rs
│   ├── mercurial.rs
│   └── traits.rs
├── monitoring/
│   ├── mod.rs
│   ├── price_monitor.rs
│   ├── yellowstone.rs
│   └── websocket.rs
├── strategy/
│   ├── mod.rs
│   ├── direct.rs
│   ├── triangular.rs
│   ├── multi_dex.rs
│   └── mev.rs
├── execution/
│   ├── mod.rs
│   ├── transaction_builder.rs
│   ├── executor.rs
│   └── jito.rs
├── math/
│   ├── mod.rs
│   ├── amm.rs
│   └── arbitrage.rs
├── risk/
│   ├── mod.rs
│   └── manager.rs
└── utils/
    ├── mod.rs
    ├── constants.rs
    └── helpers.rs
```

### Exemplo de Implementação Principal
```rust
use anyhow::Result;
use log::{info, error};
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    
    info!("Starting Solana Arbitrage Bot");
    
    // Load configuration
    let config = Arc::new(BotConfig::from_env()?);
    
    // Initialize components
    let price_monitor = Arc::new(PriceMonitor::new(config.clone()).await?);
    let strategy_engine = Arc::new(StrategyEngine::new(config.clone()));
    let execution_engine = Arc::new(ExecutionEngine::new(config.clone()).await?);
    let risk_manager = Arc::new(RiskManager::new(config.clone()));
    
    // Start price monitoring
    let monitor_handle = {
        let price_monitor = price_monitor.clone();
        tokio::spawn(async move {
            price_monitor.start().await
        })
    };
    
    // Start metrics server
    let metrics_handle = {
        let config = config.clone();
        tokio::spawn(async move {
            start_metrics_server(config.metrics_port).await
        })
    };
    
    // Main arbitrage loop
    let mut interval = interval(Duration::from_millis(config.scan_interval_ms));
    
    loop {
        interval.tick().await;
        
        // Get current prices
        let prices = price_monitor.get_current_prices().await;
        
        // Find arbitrage opportunities
        let opportunities = strategy_engine.find_opportunities(&prices).await;
        
        for opportunity in opportunities {
            // Risk check
            if !risk_manager.validate_opportunity(&opportunity).await {
                continue;
            }
            
            // Execute arbitrage
            match execution_engine.execute_arbitrage(opportunity).await {
                Ok(signature) => {
                    info!("Arbitrage executed successfully: {}", signature);
                    ARBITRAGE_SUCCESSES.inc();
                }
                Err(e) => {
                    error!("Failed to execute arbitrage: {}", e);
                }
            }
            
            ARBITRAGE_ATTEMPTS.inc();
        }
    }
}
```

---

## Segurança e Boas Práticas

### Gerenciamento de Chaves
```rust
// Use environment variables para chaves privadas
use std::env;
use solana_sdk::signature::{Keypair, read_keypair_file};

fn load_keypair() -> Result<Keypair> {
    if let Ok(key_path) = env::var("SOLANA_KEYPAIR_PATH") {
        read_keypair_file(&key_path)
            .map_err(|e| anyhow::anyhow!("Failed to read keypair: {}", e))
    } else if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        let bytes = bs58::decode(private_key)
            .into_vec()
            .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;
        Keypair::from_bytes(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create keypair: {}", e))
    } else {
        Err(anyhow::anyhow!("No keypair configuration found"))
    }
}
```

### Validação de Transações
```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::transaction::Transaction;

async fn simulate_transaction(
    client: &RpcClient,
    transaction: &Transaction,
) -> Result<bool> {
    let simulation = client.simulate_transaction(transaction).await?;
    
    if let Some(err) = simulation.value.err {
        error!("Transaction simulation failed: {:?}", err);
        return Ok(false);
    }
    
    // Check if transaction would be profitable
    let logs = simulation.value.logs.unwrap_or_default();
    let profit_check = logs.iter().any(|log| {
        log.contains("Profit check passed")
    });
    
    Ok(profit_check)
}
```

### Circuit Breakers
```rust
use std::time::{Duration, Instant};

struct CircuitBreaker {
    failure_count: u32,
    max_failures: u32,
    reset_timeout: Duration,
    last_failure: Option<Instant>,
    state: CircuitState,
}

enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() > self.reset_timeout {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }
    
    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());
        
        if self.failure_count >= self.max_failures {
            self.state = CircuitState::Open;
        }
    }
}
```

---

## Configuração e Deploy

### Arquivo .env
```bash
# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com
SOLANA_KEYPAIR_PATH=./keypair.json

# Bot Configuration
MIN_PROFIT_THRESHOLD=0.005  # 0.5%
MAX_SLIPPAGE=0.01          # 1%
MAX_POSITION_SIZE=1000000000  # 1 SOL in lamports
PRIORITY_FEE=30000         # microlamports
COMPUTE_UNIT_LIMIT=250000
SCAN_INTERVAL_MS=1000

# DEX Endpoints
RAYDIUM_API_URL=https://api.raydium.io
ORCA_API_URL=https://api.orca.so
JUPITER_API_URL=https://quote-api.jup.ag/v6
METEORA_API_URL=https://app.meteora.ag/api
PHOENIX_API_URL=https://api.phoenix.trade

# Monitoring
YELLOWSTONE_ENDPOINT=https://api.mainnet-beta.solana.com
BITQUERY_API_KEY=your_bitquery_api_key

# MEV Configuration
JITO_ENDPOINT=https://mainnet.block-engine.jito.wtf
JITO_TIP_ACCOUNT=96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5
JITO_TIP_AMOUNT=10000  # lamports

# Risk Management
MAX_DAILY_LOSS=0.05        # 5%
MAX_CONSECUTIVE_FAILURES=5
CIRCUIT_BREAKER_TIMEOUT=300  # 5 minutes

# Logging
RUST_LOG=info
LOG_FILE=./logs/arbitrage.log

# Metrics
METRICS_PORT=9090
PROMETHEUS_ENABLED=true
```

### Arquivo .gitignore
```gitignore
# Rust
/target/
**/*.rs.bk
Cargo.lock

# Environment
.env
.env.local
.env.production

# Keys and Secrets
keypair.json
*.key
*.pem
secrets/

# Logs
logs/
*.log

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Build artifacts
dist/
build/

# Dependencies
node_modules/

# Temporary files
tmp/
temp/
*.tmp

# Database
*.db
*.sqlite

# Configuration overrides
config.local.toml
settings.local.json
```

### Dockerfile
```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/solana-arbitrage-bot .

EXPOSE 9090

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

CMD ["./solana-arbitrage-bot"]
```

### Docker Compose
```yaml
version: '3.8'

services:
  arbitrage-bot:
    build: .
    environment:
      - RUST_LOG=info
    env_file:
      - .env
    volumes:
      - ./logs:/app/logs
      - ./keypair.json:/app/keypair.json:ro
    ports:
      - "9090:9090"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9090/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    depends_on:
      - prometheus

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/datasources:/etc/grafana/provisioning/datasources
    depends_on:
      - prometheus

volumes:
  grafana-storage:
  prometheus-data:
```

---

## Métricas e Monitoramento

### Prometheus Metrics
```rust
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};
use lazy_static::lazy_static;

lazy_static! {
    static ref ARBITRAGE_ATTEMPTS: Counter = register_counter!(
        "arbitrage_attempts_total",
        "Total number of arbitrage attempts"
    ).unwrap();
    
    static ref ARBITRAGE_SUCCESSES: Counter = register_counter!(
        "arbitrage_successes_total",
        "Total number of successful arbitrages"
    ).unwrap();
    
    static ref ARBITRAGE_PROFIT: Histogram = register_histogram!(
        "arbitrage_profit_percentage",
        "Profit percentage from arbitrage trades"
    ).unwrap();
    
    static ref CURRENT_BALANCE: Gauge = register_gauge!(
        "current_balance_sol",
        "Current SOL balance"
    ).unwrap();
    
    static ref PRICE_SPREAD: Histogram = register_histogram!(
        "price_spread_percentage",
        "Price spread between DEXs"
    ).unwrap();
    
    static ref TRANSACTION_FEES: Histogram = register_histogram!(
        "transaction_fees_sol",
        "Transaction fees in SOL"
    ).unwrap();
    
    static ref SLIPPAGE_OBSERVED: Histogram = register_histogram!(
        "slippage_observed_percentage",
        "Observed slippage percentage"
    ).unwrap();
}
```

### Health Check Endpoint
```rust
use warp::{Filter, Reply};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

async fn health_check() -> Result<impl Reply, warp::Rejection> {
    let health_status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": get_uptime_seconds(),
        "balance": get_current_balance().await.unwrap_or(0.0),
        "last_arbitrage": get_last_arbitrage_time().await
    });
    
    Ok(warp::reply::json(&health_status))
}

pub async fn start_metrics_server(port: u16) {
    let health = warp::path("health")
        .and(warp::get())
        .and_then(health_check);
    
    let metrics = warp::path("metrics")
        .and(warp::get())
        .map(|| {
            let encoder = prometheus::TextEncoder::new();
            let metric_families = prometheus::gather();
            encoder.encode_to_string(&metric_families).unwrap()
        });
    
    let routes = health.or(metrics);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}
```

---

## Endereços de Tokens Importantes

```rust
// Token Mints
pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const BONK_MINT: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const RAY_MINT: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
pub const ORCA_MINT: &str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";

// Program IDs
pub const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const ORCA_WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
pub const METEORA_PROGRAM_ID: &str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
pub const PHOENIX_PROGRAM_ID: &str = "PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY";
pub const SABER_PROGRAM_ID: &str = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";
pub const MERCURIAL_PROGRAM_ID: &str = "MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky";
pub const OPENBOOK_PROGRAM_ID: &str = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";
```

---

## Considerações Finais

### Performance
- Use connection pooling para RPCs
- Implemente cache para dados frequentemente acessados
- Otimize compute units para transações
- Use priority fees adequadas
- Considere usar lookup tables para reduzir tamanho de transações

### Escalabilidade
- Implemente sharding por token pairs
- Use múltiplas instâncias para diferentes estratégias
- Considere usar Jito MEV para melhor execução
- Implemente load balancing entre RPCs

### Manutenção
- Monitore logs regularmente
- Atualize SDKs e dependências
- Teste em devnet antes de deploy
- Mantenha backups das configurações
- Implemente alertas para falhas críticas

### Compliance
- Verifique regulamentações locais
- Implemente KYC se necessário
- Mantenha registros de transações
- Considere implicações fiscais
- Documente todas as estratégias utilizadas

### Riscos e Mitigações
- **Slippage**: Use simulação antes da execução
- **MEV**: Considere usar bundles Jito
- **Network congestion**: Implemente retry logic
- **Smart contract risk**: Audite contratos utilizados
- **Liquidez**: Monitore profundidade dos pools

Este documento serve como base completa para implementação de um bot de arbitragem robusto, atômico e seguro na rede Solana, cobrindo todos os aspectos técnicos necessários para o desenvolvimento e operação do sistema.