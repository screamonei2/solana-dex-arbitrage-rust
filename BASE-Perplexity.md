# BASE: Manual de Arbitragem BONK/SOL na Rede Solana

> **Propósito** – Este documento sintetiza toda a informação técnica necessária para construir, operar e manter um *bot* de arbitragem em **Rust**, focado em pares **BONK/SOL** nas principais DEX da rede **Solana**. Inclui padrões de conexão, APIs, instruções de swap, taxas, melhores práticas de segurança, algoritmo de busca de oportunidades (direta e triangular) e diretrizes de ambiente (.env, .gitignore).

---

## 1. Fundamentos da Solana e Requisitos Básicos

### 1.1 Velocidade, taxas e MEV
* Solana processa até 65 000 tps com taxas médias < 0.00001 SOL[60].  
* A ausência de mempool público reduz, mas não elimina, **MEV**; proteja bundles via Jito ou RPCs protegidos[60][61].

### 1.2 Ferramentas obrigatórias
| Ferramenta | Versão recomendada | Função |
|------------|-------------------|--------|
| Rust toolchain | stable 1.78+ | Compilação do bot |
| Solana CLI | v1.18+ | Deploy & RPC utils[80] |
| Anchor CLI | 0.31+ | Programas Rust on-chain[53] |
| Node ≥18 + Yarn | scripts & testes TypeScript[53] |
| Jito SDK (Rust) | 0.3+ | Bundles atômicos[56] |

### 1.3 Estrutura de diretórios
```
bot/
 ├─ Cargo.toml
 ├─ src/
 │   ├─ main.rs           # runner off-chain
 │   ├─ dex/
 │   │   ├─ orca.rs       # wrappers
 │   │   ├─ raydium.rs
 │   │   └─ jupiter.rs
 │   ├─ graph.rs          # Bellman-Ford
 │   └─ config.rs         # .env loader
 ├─ .env.example          # variáveis sensíveis
 └─ .gitignore            # ver seção 10
```

---

## 2. DEX que suportam BONK/SOL

| DEX | Tipo | Program ID | Pool BONK/SOL | Taxa base |
|-----|------|-----------|---------------|-----------|
| Orca (Whirlpool) | AMM CLMM | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`[14] | `3ne4mWqdYuNiYrYZC9TrA3FcfuFdErghH97vNPbjicr1`[1] | 0.30 % padrão[9] |
| Raydium CPMM v4 | AMM | `AMM4bPfX...` (legacy) | ver `fetchPools` API[33] | 0.25 %[15] |
| Raydium CLMM | Concentrated | `CLMMmWz9...` | rota via Router | 0.01-0.5 % variável[15] |
| Jupiter | Aggregator | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`[18] | N/A (rota melhor) | 0 % + taxas DEX upstream[31] |
| Meteora DLMM | AMM dinâmica | `DLMmxx...` | route `Meteora` | 0.02-0.30 % adaptativo[23] |

> **Nota**: DEX privados (Obric v2, ZeroFi) concentram 40-60 % do volume do par SOL e memecoins, mas só são acessíveis via Jupiter Router[6].

### 2.1 Conectar-se ao Orca
```rust
use orca_sdk::prelude::*;
let client = OrcaClient::new("https://api.mainnet-beta.solana.com");
let whirlpool = client.whirlpool("3ne4mWqdYuNiYrYZC9TrA3FcfuFdErghH97vNPbjicr1");
let quote = whirlpool.get_quote("BONK", 1_000_000u64, 0.5)?; // 1M BONK
let tx = whirlpool.swap(quote, &payer)?; // instrução única
```
Instrução SPL padrão: `Swap{ amount_in, sqrt_price_limit }` (8 CU ~ 130 k).

### 2.2 Conectar-se ao Raydium
* SDK V2 (`@raydium-io/raydium-sdk-v2`) ou REST `/trade`[15][21].  
* Estrutura de instrução Anchor `SwapInstruction { amount_in, min_out, pool_id }`.

### 2.3 Usar o Jupiter V6 API
```rust
// GET quote
GET https://quote-api.jup.ag/v6/quote?inputMint=So111...&outputMint=DezX...&amount=1000000&slippageBps=50
// POST swap
POST https://quote-api.jup.ag/v6/swap { quoteResponse, userPublicKey }
```
*Parâmetros chave*: `excludeDexes`, `dynamicSlippage`, `maxAccounts`, `prioritizationFeeLamports`[31].  
*Resposta*: transação serializada base64 pronta para assinar.

### 2.4 Meteora Trade Router
* GraphQL via Bitquery stream (`DEXPools`) ou SDK `meteora-rs`.  
* DLMM ajusta taxa em tempo real (volatilidade) → slippage menor[23].

---

## 3. Algoritmos de Detecção de Arbitragem

### 3.1 Arbitragem direta entre DEX
1. _Snapshot_ sincronizado de preços (`get_pool_state`).
2. Calcule `price_a`, `price_b`; se `price_a / price_b > 1 + τ` (τ = somatório de taxas + slippage)[75], gerar rota `DEX_a → DEX_b`.
3. Simule via RPC `simulateTransaction`.

### 3.2 Arbitragem triangular
Modelagem em grafo orientado: vértices = mints, arestas = `−ln(rate * (1−fee))`.  
A existência de ciclo negativo ⇒ lucro[79][89].
```rust
// Bellman-Ford (simplificado)
for _ in 0..n-1 {
  for e in edges.iter() {
     if dist[e.u] + e.w < dist[e.v] {
        dist[e.v] = dist[e.u] + e.w;
        parent[e.v] = e.u;
     }
  }
}
// ciclo? dist[u] + w < dist[v]
```
Após detectar ciclo, reconstrua ordem `SOL → USDC → BONK → SOL` conforme case study (95 SOL lucro bruto)[77].

### 3.3 Dimensionamento ótimo de trade
Itere montante até condição `∏ rates = 1 + Σ fees`[84].  
Use busca binária para máximo `amount_in` que ainda retorna `profit > 0`.

---

## 4. Arquitetura do Bot

1. **Price-Watcher** – websocket RPC ou Pine Analytics feed; cadastra pools BONK/SOL.  
2. **Graph Builder** – atualiza pesos e roda Bellman-Ford em thread assíncrona.  
3. **Route Simulator** – usa `simulateTransaction` + `computeUnitLimit=1_400_000`.  
4. **Bundle Executor** – agrupa instruções (até 5) em Jito bundle com tip prioritário[56].  
5. **Risk Manager** – verifica `slippage`, `min_out`, `balance_after > before`.  
6. **Logger** – exporta métricas Prometheus.

Diagrama de sequência:
```
Watcher ──prices──▶ Graph │
Graph ──cycle────▶ Simulator ──OK──▶ Executor ──signed tx──▶ Jito BE
                             │                        ▲
                             └──fail────────────error──┘
```

---

## 5. Boas Práticas de Segurança
* **Ownership check** em todas contas de programa[58][63].  
* Use `checked_add/sub/mul` para evitar overflow[63].  
* Trate erros com `thiserror` + `anyhow`; evite `unwrap()` exceto testes[62][57].  
* MEV protection: bundles + slippage dynamic + endpoints protegidos[60][71].  
* Valide oráculos (Pyth) para health factors em rotas com empréstimo.

---

## 6. Integração RPC & Prioridade
```rust
let rpc = RpcClient::new_with_commitment(RPC_URL, CommitmentConfig::confirmed());
let fee = rpc.get_fee_for_message(&msg)?;
let priority_fee = compute_dynamic_fee(&cluster_stats);
```
* Use múltiplos endpoints + fallback (QuickNode, Helius, Anza)[45][92].  
* Attach `prioritizationFeeLamports` no Jupiter ou compute budget IX.

---

## 7. Variáveis de Ambiente (.env)
```ini
RPC_URL="https://mainnet.helius.rpc/…"
JITO_ENGINE="https://mainnet.block-engine.jito.wtf/api/v1/bundle"
PRIVATE_KEY="<base58>"
PUBLIC_KEY="<wallet>"
MAX_SLIPPAGE_BPS=50
MAX_ACCOUNTS=54
DEX_EXCLUDE="ObricV2,ZeroFi"
```
Carregue com `dotenvy::dotenv()`[64] ou `rust_dotenv`[59].

---

## 8. Template .gitignore (Rust + Solana)
```
# Rust
/target/
**/*.rs.bk
Cargo.lock

# Solana & Anchor
**/.anchor/
**/program/target/
**/program/idl.json

# Keys & env
*.json
!.anchor/*.json
.env*

# Logs
*.log
```
Referência: `solana-rust-examples/.gitignore`[96] + discussão cargo[81][86].

---

## 9. Deploy & Test
1. `anchor build && anchor test` – usa devnet local[85].
2. Envie programa wrapper opcional (`program/arb_router`) e fixe `PROGRAM_ID` no .env.  
3. Use QuickNode devnet faucet para SOL inicial.

---

## 10. Referências cruciais
[1] GeckoTerminal Orca pool. [6] FXStreet análise DEX privados. [9] Orca SDK docs. [14] Bitquery Orca API. [15] Raydium docs. [18] Bitquery Jupiter API. [23] Meteora DEX artigo. [31] Jupiter Station API. [56] QuickNode Jito Bundles guia. [60] QuickNode MEV overview. [63] Slowmist best practices. [67] Rust Book error handling. [75] Reddit slippage explainer. [77] EigenPhi triangular case. [79] ITB Bellman-Ford paper. [80] Solana Quick Start. [81] Gitignore cargo thread. [92] InstantNodes RPC integration.
