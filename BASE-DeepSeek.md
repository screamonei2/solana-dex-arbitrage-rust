## BASE.md: Guia para Construção de Bot de Arbitragem em Solana (Rust)  

---

### **1. Visão Geral do Projeto**  
Desenvolver um bot de arbitragem atômica em Rust para a rede Solana, focado em pares **BONK/SOL** e estratégias triangulares (ex: `SOL → BONK → USDC → SOL`). Requisitos:  
- **Atomicidade**: Transações devem ser concluídas em um único bloco ou revertidas.  
- **Segurança**: Uso de variáveis de ambiente (`.env`) e `gitignore` para chaves privadas.  
- **Eficiência**: Baixa latência (<400 ms) e custo otimizado .  

---

### **2. DEXs Solana com Par BONK/SOL**  
Principais DEXs a integrar (baseado em volume e liquidez) :  
1. **Jupiter (Agregador)**:  
   - Volume diário: $900M.  
   - API: [`jup.ag`](https://jup.ag) (suporte a swaps atômicos via CPI).  
   - Endpoint: `https://quote-api.jup.ag/v6/quote?inputMint=SOL&outputMint=BONK&amount=1000000`.  
2. **Raydium**:  
   - Liquidez: ~$500M.  
   - Programa: `RAYDIUM_LIQUIDITY_POOL_V4` (ID: `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp`).  
3. **Orca**:  
   - Par CLMM: `BONK/SOL` (TVL: $360M).  
   - SDK: `@orca-so/whirlpools-sdk` .  
4. **Lifinity**:  
   - Modelo PMM (menor perda impermanente).  

---

### **3. APIs e Dados de Mercado**  
Fontes para detecção de oportunidades:  
- **Bitquery**:  
  - Consulta em tempo real:  
    ```graphql  
    subscription { Solana { DEXTrades( where: {Trade: {Currency: {MintAddress: {is: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"}}}) { ... } } }  
    ``` .  
- **CoinGecko On-Chain API**:  
  - Endpoint: `/onchains/network/solana/pools` (preços, liquidez, volume) .  
- **DEX Screener**: Monitorar pares `BONK/SOL` em tempo real .  

---

### **4. Estratégias de Arbitragem**  
#### **4.1 Arbitragem Direta**  
- **Fluxo**: `SOL → BONK → SOL` (via Jupiter ou Raydium).  
- **Condição**: Diferença > 0.5% entre DEXs + taxas.  

#### **4.2 Arbitragem Triangular**  
- **Exemplo**:  
  1. Comprar `BONK` com `SOL` na Orca.  
  2. Vender `BONK` por `USDC` na Raydium.  
  3. Vender `USDC` por `SOL` na Jupiter .  
- **Cálculo de Viabilidade**:  
  ```rust  
  let profit = (sol_final - sol_initial) - fees;  
  if profit > MIN_PROFIT { execute_swap(); }  
  ```  

---

### **5. Implementação em Rust**  
#### **5.1 Dependências Críticas (`Cargo.toml`)**  
```toml  
[dependencies]  
solana-client = "1.18.0"  
solana-sdk = "1.18.0"  
reqwest = "0.11.0"  # Para APIs  
serde = { version = "1.0", features = ["derive"] }  
dotenv = "0.15.0"  
```  

#### **5.2 Configuração Inicial (`main.rs`)**  
```rust  
use std::env;  
use solana_client::rpc_client::RpcClient;  
use solana_sdk::signature::{Keypair, Signer};  

#[tokio::main]  
async fn main() {  
    dotenv::dotenv().ok();  
    let rpc_url = env::var("RPC_URL").expect("RPC_URL not set");  
    let keypair = Keypair::from_base58_string(&env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set"));  
    let client = RpcClient::new(rpc_url);  
}  
```  

#### **5.3 Execução de Swap Atômico (via Jupiter)**  
```rust  
async fn execute_swap(client: &RpcClient, keypair: &Keypair, route: &Route) -> Result<Signature, Box<dyn std::error::Error>> {  
    let swap_instruction = jupiter_api::swap_instruction(  
        route,  
        &keypair.pubkey(),  
    )?;  
    let transaction = Transaction::new_signed_with_payer(  
        &[swap_instruction],  
        Some(&keypair.pubkey()),  
        &[keypair],  
        client.get_latest_blockhash()?,  
    );  
    client.send_and_confirm_transaction(&transaction)?;  
}  
```  

---

### **6. Taxas e Custos**  
- **Taxa de Rede Solana**: ~0.00064 SOL/tx (base) .  
- **Taxas de DEX**: 0.1–0.3% por swap.  
- **Custo MEV**: Até 90% do lucro em operações competitivas .  
- **Orçamento Mínimo**: 0.02 SOL para inicialização + 0.005 SOL/tx.  

---

### **7. Mitigação de Riscos**  
- **MEV Protection**:  
  - Usar `priority_fee` alto (ex: 10-100 microlamports/unit) para evitar frontrunning .  
  - Modo "MEV Secure" (inspirado no BONKbot) .  
- **Tratamento de Erros**:  
  ```rust  
  match execute_swap(...).await {  
      Ok(sig) => log::info!("Swap confirmed: {}", sig),  
      Err(e) => log::error!("Failed: {:?}", e),  
  };  
  ```  
- **Segurança de Chaves**:  
  ```.gitignore  
  .env  
  target/  
  ```  
  Arquivo `.env`:  
  ```  
  RPC_URL="https://api.mainnet-beta.solana.com"  
  PRIVATE_KEY="your_base58_private_key"  
  ```  

---

### **8. Otimizações**  
- **Gas Golfing**:  
  - Endereços curtos (ex: `0x0000...`) para reduzir dados na TX .  
  - Pré-computar rotas para minimizar operações on-chain.  
- **Concorrência**:  
  ```rust  
  let tasks: Vec<_> = opportunities.into_iter().map(|opp| tokio::spawn(execute_arb(opp))).collect();  
  futures::future::join_all(tasks).await;  
  ```  

---

### **9. Testes e Simulação**  
- **Testnet**: Usar `https://api.testnet.solana.com`.  
- **Simulação de TX**:  
  ```rust  
  let result = client.simulate_transaction(&transaction)?;  
  assert!(result.value.is_success());  
  ```  
- **Ferramentas**:  
  - `solana-test-validator` para ambiente local.  
  - `solscan.io` para debug de transações .  

---

### **10. Referências**  
1. [Documentação Jupiter API](https://docs.jup.ag)  
2. [Bitquery: Solana DEX Trades](https://bitquery.io)   
3. [CoinGecko DEX API](https://www.coingecko.com/es/api/dex)   
4. [MEV Protection Strategies](https://ethereum.org/pt-br/developers/docs/mev/)   

---  
**Nota**: Atualize este documento conforme novas DEXs, APIs ou vulnerabilidades forem identificadas. Use `solana-cli` para deploy e monitoramento.