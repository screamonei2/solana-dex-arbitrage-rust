# 🚀 Solana Arbitrage Bot - BONK/SOL (2024 Edition)

> **Bot de arbitragem automatizado para pares BONK/SOL na rede Solana com atualizações críticas para 2024**

[![Rust](https://img.shields.io/badge/Rust-1.88+-orange.svg)](https://www.rust-lang.org/)
[![Solana](https://img.shields.io/badge/Solana-1.18+-blue.svg)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Anchor-0.31+-purple.svg)](https://www.anchor-lang.com/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## ⚠️ ATUALIZAÇÕES CRÍTICAS 2024

### 🔴 **MUDANÇAS OBRIGATÓRIAS IMPLEMENTADAS**

- **✅ RPC Provider Profissional**: Helius/QuickNode/Alchemy obrigatório (endpoints públicos bloqueados)
- **✅ MEV Protection Atualizada**: Priority fees dinâmicas (Jito mempool suspenso março 2024)
- **✅ Rate Limiting Avançado**: Sistema baseado no plano do provider RPC
- **✅ Failover RPC Automático**: Health checks e switching automático
- **✅ Ferramentas Atualizadas**: Rust 1.88+, Anchor 0.31+, Solana CLI 1.18+

## 🛡️ Características de Segurança 2024

- **Priority Fees Dinâmicas**: Ajuste automático baseado em congestionamento da rede
- **Rate Limiting Inteligente**: Proteção contra 429 errors de RPC providers
- **Validação Rigorosa**: Verificação obrigatória de configurações críticas
- **Circuit Breakers**: Proteção automática contra perdas excessivas
- **Monitoring 24/7**: Prometheus + Grafana com alertas em tempo real

## 📋 Pré-requisitos Obrigatórios

### 🔧 Ferramentas (Versões Mínimas)
```bash
# Rust toolchain ATUALIZADO
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
rustc --version  # Deve ser >= 1.88.0

# Solana CLI ATUALIZADO
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"
solana --version  # Deve ser >= 1.18.0

# Anchor Framework ATUALIZADO
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
anchor --version  # Deve ser >= 0.31.0
```

### 💰 RPC Provider Profissional (OBRIGATÓRIO)

**⚠️ CRÍTICO**: Endpoints públicos são inadequados para produção.

| Provider | Free Tier | Paid Plans | Rate Limits | Recomendação |
|----------|-----------|------------|-------------|--------------|
| **Helius** | 500K credits | $49/mês (10M) | 500 RPS | ⭐ **Melhor para Solana** |
| **QuickNode** | 10M credits | $49/mês (20M) | 500 RPS | ⭐ **Boa alternativa** |
| **Alchemy** | 12M transactions | $49/mês (16M) | 120 RPS | ✅ **Opção sólida** |

### 💸 Capital Mínimo Recomendado
- **Desenvolvimento**: 0.5 SOL (~$50)
- **Produção**: 10+ SOL (~$1000) para oportunidades significativas

## 🚀 Instalação e Configuração

### 1. Clone e Build
```bash
git clone <repository-url>
cd solana-arbitrage-bot
cargo build --release
```

### 2. Configuração de Ambiente

Crie `.env` baseado no template:

```bash
# RPC Configuration - OBRIGATÓRIO PROVIDER PROFISSIONAL
ARB_BOT_RPC_PRIMARY_URL="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
ARB_BOT_RPC_BACKUP_URLS="https://api.mainnet-beta.solana.com,https://solana-api.projectserum.com"
ARB_BOT_RPC_RATE_LIMIT_RPS=500
ARB_BOT_RPC_MAX_CONCURRENT_REQUESTS=10

# MEV Protection - ATUALIZADO 2024
ARB_BOT_MEV_PROTECTION_JITO_ENABLED=false  # Mempool público suspenso
ARB_BOT_MEV_PROTECTION_PRIORITY_FEE_DYNAMIC=true  # OBRIGATÓRIO
ARB_BOT_MEV_PROTECTION_SLIPPAGE_ADAPTIVE=true

# Trading Parameters
ARB_BOT_TRADING_BASE_PRIORITY_FEE=30000
ARB_BOT_TRADING_MAX_PRIORITY_FEE_LAMPORTS=50000
ARB_BOT_TRADING_MIN_PROFIT_THRESHOLD=0.005  # 0.5%

# Rate Limiting - NOVO 2024
ARB_BOT_RATE_LIMITING_REQUESTS_PER_SECOND=100
ARB_BOT_RATE_LIMITING_MAX_CONCURRENT_REQUESTS=10

# Failover - NOVO 2024
ARB_BOT_FAILOVER_HEALTH_CHECK_INTERVAL_SECONDS=30
ARB_BOT_FAILOVER_THRESHOLD_MS=5000

# Security
ARB_BOT_SECURITY_STRICT_INPUT_VALIDATION=true
ARB_BOT_SECURITY_SAFE_MATH_OPERATIONS=true

# Wallet (NUNCA commitar chaves reais)
ARB_BOT_WALLET_PRIVATE_KEY="<base58_private_key>"
ARB_BOT_WALLET_PUBLIC_KEY="<wallet_address>"
```

### 3. Validação de Configuração

```bash
# Verificar configuração
cargo run --bin validate-config

# Testar conectividade RPC
cargo run --bin test-rpc
```

## 🏃 Execução

### Desenvolvimento
```bash
# Com logs detalhados
RUST_LOG=debug cargo run

# Com rate limiting reduzido para testes
ARB_BOT_RATE_LIMITING_REQUESTS_PER_SECOND=10 cargo run
```

### Produção
```bash
# Build otimizado
cargo build --release

# Executar em background
nohup ./target/release/solana-arbitrage-bot > bot.log 2>&1 &
```

### Docker (Recomendado para Produção)
```bash
# Build
docker build -t arbitrage-bot .

# Run com configuração
docker run -d \
  --name arbitrage-bot \
  --env-file .env \
  -p 9090:9090 \
  arbitrage-bot
```

## 📊 Monitoramento e Métricas

### Endpoints Disponíveis
- **Métricas**: `http://localhost:9090/metrics` (Prometheus)
- **Health Check**: `http://localhost:9090/health`
- **Estatísticas**: `http://localhost:9090/stats`

### Métricas Principais
- `arbitrage_attempts_total`: Total de tentativas
- `arbitrage_successes_total`: Arbitragens bem-sucedidas  
- `rpc_request_duration_seconds`: Latência RPC
- `priority_fee_lamports`: Priority fees atuais
- `wallet_balance_sol`: Saldo da carteira

### Dashboards Grafana

Importar dashboards pré-configurados:
- `grafana/arbitrage-dashboard.json`: Dashboard principal
- `grafana/technical-dashboard.json`: Métricas técnicas
- `grafana/risk-dashboard.json`: Monitoramento de riscos

## 🛠️ Arquitetura (Atualizada 2024)

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Rate Limiter    │◄──►│ RPC Provider     │◄──►│ Solana Network  │
│ (Novo 2024)     │    │ (Profissional)   │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                        │                       │
         ▼                        ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Priority Fee    │◄──►│ Price Monitor    │◄──►│ DEX APIs        │
│ Calculator      │    │                  │    │ (Raydium/Orca)  │
│ (Novo 2024)     │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                        │                       │
         ▼                        ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Strategy Engine │◄──►│ Execution Engine │◄──►│ Risk Manager    │
│                 │    │                  │    │ (Circuit Break) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Componentes Principais

1. **Rate Limiter**: Controla requests para evitar 429 errors
2. **Priority Fee Calculator**: Calcula fees dinâmicos baseado em congestionamento  
3. **Price Monitor**: Monitora preços via WebSocket/gRPC
4. **Strategy Engine**: Detecta oportunidades de arbitragem
5. **Execution Engine**: Executa trades com proteção MEV
6. **Risk Manager**: Circuit breakers e validação de riscos

## 🎯 Estratégias Implementadas

### 1. Arbitragem Direta
- **SOL → BONK** (DEX A) → **SOL** (DEX B)
- Aproveita diferenças de preço diretas
- Execução rápida (2 transações)

### 2. Arbitragem Triangular  
- **SOL → BONK → USDC → SOL**
- Múltiplas oportunidades por ciclo
- Maior complexidade, maior potencial

### 3. Multi-DEX Routing
- Utiliza Jupiter para roteamento otimizado
- Acesso a DEXs privados (40-60% do volume)
- Smart routing adaptativo

## ⚡ Performance e Otimizações 2024

### Melhorias Implementadas
- **Async Architecture**: Non-blocking I/O
- **Connection Pooling**: Múltiplas conexões RPC  
- **Batch Operations**: Agrupamento de requests
- **Smart Caching**: Cache inteligente com TTL
- **Parallel Processing**: Consultas simultâneas a DEXs

### Benchmarks (Hardware Médio)
- **Latência de Detecção**: < 100ms
- **Execução de Trade**: < 2 segundos
- **Throughput**: 100+ oportunidades/minuto analisadas
- **Uptime**: 99.9%+ com failover RPC

## 🚨 Alertas e Monitoramento

### Alertas Críticos Configurados
- **High Failure Rate**: > 10% em 5 minutos
- **Low Profitability**: < 0.1% por hora  
- **High RPC Latency**: > 2 segundos no p95
- **Low Wallet Balance**: < 0.1 SOL
- **Circuit Breaker**: Ativação automática

### Integração de Alertas
- **Discord**: Webhook para alertas críticos
- **Slack**: Métricas de performance
- **Email**: Relatórios diários
- **PagerDuty**: Incidentes críticos

## 🛡️ Segurança e Compliance

### Medidas de Segurança Implementadas
- **Private Key Management**: HSM/Vault em produção
- **Input Validation**: Validação rigorosa de todos os inputs
- **Safe Math**: Operações matemáticas protegidas contra overflow
- **Circuit Breakers**: Proteção automática contra perdas
- **Rate Limiting**: Proteção contra abuse de APIs
- **Audit Logging**: Trilha completa de auditoria

### Compliance
- **Verificar regulamentações locais** antes do deploy
- **KYC/AML**: Implementar se necessário para sua jurisdição
- **Backup de Dados**: Políticas de retenção e backup
- **GDPR/LGPD**: Proteção de dados pessoais se aplicável

## 🧪 Testes

### Executar Testes
```bash
# Testes unitários
cargo test

# Testes de integração  
cargo test --test integration

# Testes com RPC real (devnet)
RUST_LOG=debug cargo test --test rpc_tests -- --ignored

# Testes de performance
cargo test --release --test benchmark
```

### Coverage
```bash
# Instalar ferramentas
cargo install cargo-tarpaulin

# Gerar relatório de coverage
cargo tarpaulin --out Html
```

## 🐛 Troubleshooting

### Problemas Comuns

#### 1. "RPC Provider Validation Failed"
```bash
# Verificar se está usando provider profissional
echo $ARB_BOT_RPC_PRIMARY_URL

# Testar conectividade
curl -X POST $ARB_BOT_RPC_PRIMARY_URL \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

#### 2. "Rate Limit Exceeded"
```bash
# Verificar configuração
echo $ARB_BOT_RATE_LIMITING_REQUESTS_PER_SECOND

# Aumentar se necessário (baseado no plano)
export ARB_BOT_RATE_LIMITING_REQUESTS_PER_SECOND=200
```

#### 3. "Priority Fee Too High"
```bash
# Verificar configuração de fees
echo $ARB_BOT_TRADING_MAX_PRIORITY_FEE_LAMPORTS

# Ajustar se necessário
export ARB_BOT_TRADING_MAX_PRIORITY_FEE_LAMPORTS=100000
```

#### 4. "MEV Protection Deprecated"
```bash
# Desabilitar Jito (mempool público suspenso)
export ARB_BOT_MEV_PROTECTION_JITO_ENABLED=false

# Habilitar priority fees dinâmicas
export ARB_BOT_MEV_PROTECTION_PRIORITY_FEE_DYNAMIC=true
```

### Logs e Debugging
```bash
# Logs detalhados
RUST_LOG=debug cargo run

# Apenas erros
RUST_LOG=error cargo run

# Logs específicos do módulo
RUST_LOG=solana_arbitrage_bot::rate_limiting=debug cargo run
```

## 📚 Documentação Adicional

- **[API Reference](docs/api.md)**: Documentação completa da API
- **[Configuration Guide](docs/config.md)**: Guia detalhado de configuração
- **[Security Guide](docs/security.md)**: Práticas de segurança
- **[Deployment Guide](docs/deployment.md)**: Deploy em produção
- **[Troubleshooting Guide](docs/troubleshooting.md)**: Resolução de problemas

## 🤝 Contribuição

1. Fork o projeto
2. Crie uma branch para feature (`git checkout -b feature/nova-dex`)
3. Commit suas mudanças (`git commit -am 'Adiciona suporte a Nova DEX'`)
4. Push para a branch (`git push origin feature/nova-dex`)
5. Abra um Pull Request

### Diretrizes de Contribuição
- Seguir [dev_rules.md](dev_rules.md) para padrões de código
- Testes obrigatórios para novas funcionalidades
- Documentação atualizada
- Code review aprovado

## 📄 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## ⚠️ Disclaimer

**AVISO IMPORTANTE**: Trading automatizado envolve riscos significativos. Este software é fornecido "como está", sem garantias. Use por sua própria conta e risco.

- **Não é aconselhamento financeiro**
- **Teste thoroughly em devnet antes de usar fundos reais**  
- **Monitore constantemente em produção**
- **Mantenha fundos de reserva para emergências**
- **Verifique regulamentações locais**

## 📞 Suporte

- **Issues**: [GitHub Issues](https://github.com/seu-repo/issues)
- **Discussions**: [GitHub Discussions](https://github.com/seu-repo/discussions)
- **Discord**: [Solana Arbitrage Community](https://discord.gg/...)

---

## 🎯 Roadmap 2024

### Q1 2024 ✅
- [x] Priority fees dinâmicas
- [x] Rate limiting avançado  
- [x] RPC provider profissional
- [x] Failover automático

### Q2 2024 🔄
- [ ] Machine Learning para predição de oportunidades
- [ ] Suporte a mais pares de tokens
- [ ] WebUI para monitoramento
- [ ] API REST para controle externo

### Q3-Q4 2024 📋
- [ ] Suporte a Solana 2.0
- [ ] Token Extensions support
- [ ] Cross-chain arbitrage (Wormhole)
- [ ] Advanced MEV strategies

---

**Construído com ❤️ para a comunidade Solana**

*Última atualização: Janeiro 2025* 