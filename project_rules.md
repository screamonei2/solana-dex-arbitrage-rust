# PROJECT RULES - Bot de Arbitragem BONK/SOL Solana

*Regras de projeto baseadas no manual consolidado ALL_BASE.md*

## ⚠️ PRIORIDADES CRÍTICAS 2024

**IMPLEMENTAR OBRIGATORIAMENTE ANTES DE PRODUÇÃO:**

### 🔴 Alta Prioridade (Implementar Primeiro)
1. **RPC Provider Profissional** - Helius, QuickNode ou Alchemy
2. **MEV Protection Atualizada** - Priority fees dinâmicas (Jito mempool suspenso)
3. **Rate Limiting** - Controle baseado no plano do provider
4. **Failover RPC** - Sistema de backup automático
5. **Rust 1.88+ e Anchor 0.31+** - Ferramentas atualizadas

### 🟡 Média Prioridade (Implementar Segundo)
1. **Monitoring Avançado** - Prometheus + Grafana com alertas
2. **Testing Robusto** - Anchor test framework
3. **Security Hardening** - Circuit breakers atualizados
4. **Solana Kit Integration** - Para componentes JavaScript
5. **Documentation** - Runbooks e procedimentos

---

## 1. Regras de Arquitetura

### 1.1 Ferramentas Modernas (Obrigatório 2024)
**Referência**: [8.6 Desenvolvimento Moderno com Solana](ALL_BASE.md#86-desenvolvimento-moderno-com-solana-2024)

**1.1.a** **Rust Toolchain Atualizado**
- Rust 1.88+ obrigatório para melhores práticas
- Anchor Framework 0.31+ com melhor performance
- Solana CLI 1.18+ com novos recursos
- Cargo features para otimização de build

**1.1.b** **Solana Kit Integration**
- Usar Solana Kit para operações JavaScript quando aplicável
- Integração com Web3.js v2 para melhor performance
- TypeScript strict mode obrigatório
- ESLint e Prettier configurados

**1.1.c** **Ferramentas de Desenvolvimento**
- Anchor test framework para testes robustos
- Solana-test-validator para testes locais
- Metaplex CLI para operações NFT (se aplicável)
- Jupiter SDK v6 para agregação

### 1.2 Estrutura de Código
**Referência**: [6.2 Estrutura do Projeto](ALL_BASE.md#62-estrutura-do-projeto)

**1.2.a** **Modularidade Obrigatória**
- Cada DEX deve ter seu próprio módulo separado
- Implementar trait `DexClient` para padronização [4.1.b](ALL_BASE.md#41b-interface-dex-client)
- Separar lógica de negócio da implementação específica

**1.2.b** **Separação de Responsabilidades**
- `src/dex/`: Implementações específicas de cada DEX
- `src/arbitrage/`: Estratégias de arbitragem
- `src/monitoring/`: Monitoramento e métricas
- `src/security/`: Validações e circuit breakers

**1.2.c** **Configuração Centralizada**
- Todas as configurações devem estar em `config/`
- Usar variáveis de ambiente conforme [9.1](ALL_BASE.md#91-variáveis-de-ambiente-env)
- Nunca hardcodar valores sensíveis no código

### 1.3 Padrões de Design
**Referência**: [4.1 Componentes Principais](ALL_BASE.md#41-componentes-principais)

**1.2.a** **Factory Pattern para DEXs**
- Implementar factory para criação de clientes DEX
- Permitir configuração dinâmica de DEXs ativas
- Facilitar adição de novas DEXs sem modificar código core

**1.2.b** **Strategy Pattern para Arbitragem**
- Cada estratégia deve implementar trait comum
- Permitir execução paralela de múltiplas estratégias
- Configuração via arquivo de estratégias ativas

**1.2.c** **Observer Pattern para Monitoramento**
- Eventos de preço devem notificar múltiplos observadores
- Métricas devem ser coletadas de forma não-bloqueante
- Logs estruturados para todas as operações

## 2. Regras de Segurança

### 2.1 Gerenciamento de Chaves
**Referência**: [8.2 Gerenciamento de Chaves](ALL_BASE.md#82-gerenciamento-de-chaves)

**2.1.a** **Armazenamento Seguro**
- NUNCA commitar chaves privadas no repositório
- Usar variáveis de ambiente para todas as chaves
- Implementar rotação automática de chaves quando possível

**2.1.b** **Validação de Chaves**
- Validar formato e validade das chaves na inicialização
- Implementar fallback para chaves de backup
- Log de tentativas de acesso não autorizadas

**2.1.c** **Permissões Mínimas**
- Usar chaves com permissões mínimas necessárias
- Separar chaves para diferentes ambientes (dev/prod)
- Implementar rate limiting para operações sensíveis

### 2.2 Validação de Transações
**Referência**: [8.3 Validação de Transações](ALL_BASE.md#83-validação-de-transações)

**2.2.a** **Validação Pré-Execução**
- Simular todas as transações antes da execução
- Validar saldos suficientes para todas as operações
- Verificar slippage máximo permitido

**2.2.b** **Validação Pós-Execução**
- Confirmar que o resultado esperado foi alcançado
- Verificar que não houve perda inesperada de fundos
- Log detalhado de todas as transações executadas

**2.2.c** **Circuit Breakers**
**Referência**: [8.4 Circuit Breakers](ALL_BASE.md#84-circuit-breakers)
- Implementar limites de perda máxima por período
- Pausar operações em caso de comportamento anômalo
- Alertas automáticos para situações de risco

## 3. Regras de Performance

### 3.1 Otimização de Latência
**Referência**: [10.1 Performance e Escalabilidade](ALL_BASE.md#101-performance-e-escalabilidade)

**3.1.a** **Arquitetura Assíncrona**
- Todas as operações I/O devem ser assíncronas
- Usar connection pooling para RPC endpoints
- Implementar timeout adequado para todas as operações

**3.1.b** **Caching Inteligente**
- Cache de dados de pools que mudam pouco
- Invalidação automática baseada em eventos
- TTL apropriado para diferentes tipos de dados

**3.1.c** **Paralelização**
- Consultar múltiplas DEXs em paralelo
- Processar oportunidades de arbitragem concorrentemente
- Usar thread pools para operações CPU-intensivas

### 3.2 Gestão de Recursos
**3.2.a** **RPC Providers Profissionais (Obrigatório 2024)**
**Referência**: [1.4 RPC Providers para Produção](ALL_BASE.md#14-rpc-providers-para-produção)
- **NUNCA** usar endpoints públicos em produção
- Configurar provider principal: Helius (recomendado), QuickNode ou Alchemy
- Rate limits: mínimo 500 RPS para arbitragem
- SLA de uptime: mínimo 99.9%
- Configurar múltiplos providers para failover

**3.2.b** **Limites de Conexão**
- Máximo de conexões simultâneas por RPC endpoint
- Rate limiting baseado no plano do provider
- Fallback automático entre RPC providers
- Health checks a cada 30 segundos
- Timeout de 5 segundos para detectar falhas

**3.2.c** **Gestão de Memória**
- Evitar vazamentos de memória em loops longos
- Limpeza periódica de caches antigos
- Monitoramento de uso de memória

**3.2.d** **Monitoramento de Performance**
**Referência**: [9.5 Métricas Prometheus](ALL_BASE.md#95-métricas-prometheus)
- Métricas de latência para todas as operações
- Alertas para degradação de performance
- Dashboards em tempo real

## 4. Regras de Integração

### 4.1 Integração com DEXs
**Referência**: [2. DEXs que Suportam BONK/SOL](ALL_BASE.md#2-dexs-que-suportam-bonksol)

**4.1.a** **Padronização de Interface**
- Todas as DEXs devem implementar o mesmo trait
- Normalização de formatos de resposta
- Tratamento uniforme de erros

**4.1.b** **Versionamento de APIs**
- Suporte a múltiplas versões de API quando necessário
- Migração gradual para novas versões
- Fallback para versões anteriores em caso de falha

**4.1.c** **Tratamento de Erros**
- Retry automático com backoff exponencial
- Classificação de erros (temporários vs permanentes)
- Logging detalhado de todos os erros

### 4.2 Integração MEV (Atualizado 2024)
**Referência**: [7.3 Jito MEV Integration](ALL_BASE.md#73-jito-mev-integration) e [7.4 Alternativas ao Jito](ALL_BASE.md#74-alternativas-ao-jito-mempool)

**4.2.a** **Status Jito (Março 2024)**
- Jito suspendeu mempool público - usar apenas com private bundles
- Tip mínimo obrigatório: 10.000 lamports
- Configurar JITO_ENABLED=false por padrão
- Implementar fallback para priority fees dinâmicas

**4.2.b** **MEV Protection Alternativa**
- Priority fees dinâmicas baseadas em congestionamento
- Slippage adaptativo durante alta volatilidade
- Transaction splitting para trades grandes
- Timing optimization para evitar horários de pico MEV
- Considerar AMMs resistentes a sandwich (ex: Plasma)

**4.2.c** **Bundle Management (Quando Aplicável)**
- Usar apenas private bundles com tip mínimo
- Fallback obrigatório para transações individuais
- Monitorar custos vs benefícios dos bundles

## 5. Regras de Monitoramento

### 5.1 Logging
**5.1.a** **Estrutura de Logs**
- Usar formato JSON estruturado
- Incluir timestamp, nível, componente e contexto
- Correlação de logs por transaction ID

**5.1.b** **Níveis de Log**
- ERROR: Falhas que impedem operação
- WARN: Situações que requerem atenção
- INFO: Operações importantes do sistema
- DEBUG: Informações detalhadas para troubleshooting

**5.1.c** **Retenção de Logs**
- Logs de produção: 30 dias mínimo
- Logs de debug: 7 dias
- Logs de auditoria: 1 ano

### 5.2 Métricas
**Referência**: [9.5 Métricas Prometheus](ALL_BASE.md#95-métricas-prometheus)

**5.2.a** **Métricas de Negócio**
- Número de oportunidades detectadas
- Taxa de sucesso de arbitragem
- Profit/Loss por período
- Volume total processado

**5.2.b** **Métricas Técnicas**
- Latência de operações
- Taxa de erro por componente
- Uso de recursos (CPU, memória, rede)
- Disponibilidade de RPC endpoints

**5.2.c** **Alertas**
- Perda acima do limite configurado
- Falhas consecutivas de transação
- Degradação de performance
- Indisponibilidade de DEXs críticas

## 6. Regras de Deploy

### 6.1 Ambientes
**Referência**: [9.4 Deploy & Test](ALL_BASE.md#94-deploy--test)

**6.1.a** **Separação de Ambientes**
- Development: Devnet Solana, dados mock
- Staging: Testnet Solana, dados reais limitados
- Production: Mainnet Solana, operação completa

**6.1.b** **Configuração por Ambiente**
- Arquivos de configuração separados
- Variáveis de ambiente específicas
- Limites de risco diferentes por ambiente

**6.1.c** **Processo de Deploy**
- Testes automatizados obrigatórios
- Deploy gradual com rollback automático
- Validação pós-deploy obrigatória

### 6.2 Versionamento
**6.2.a** **Semantic Versioning**
- MAJOR.MINOR.PATCH format
- Breaking changes incrementam MAJOR
- Features incrementam MINOR
- Bug fixes incrementam PATCH

**6.2.b** **Release Notes**
- Documentar todas as mudanças
- Incluir instruções de migração se necessário
- Destacar mudanças de configuração

## 7. Regras de Teste

### 7.1 Cobertura de Testes
**7.1.a** **Cobertura Mínima**
- 80% de cobertura de código obrigatória
- 100% de cobertura para funções críticas
- Testes de integração para todos os componentes

**7.1.b** **Tipos de Teste**
- Unit tests: Lógica individual
- Integration tests: Interação entre componentes
- End-to-end tests: Fluxo completo de arbitragem
- Performance tests: Latência e throughput

### 7.2 Estratégia de Teste
**7.2.a** **Test-Driven Development**
- Escrever testes antes da implementação
- Refatorar com confiança
- Documentação viva através de testes

**7.2.b** **Mocking e Stubbing**
- Mock de APIs externas em testes unitários
- Dados de teste realistas
- Simulação de cenários de erro

## 8. Regras de Documentação

### 8.1 Documentação de Código
**8.1.a** **Comentários**
- Explicar o "porquê", não o "como"
- Documentar decisões de design
- Atualizar comentários junto com código

**8.1.b** **Documentação de API**
- Documentar todas as funções públicas
- Incluir exemplos de uso
- Especificar comportamento em casos de erro

### 8.2 Documentação de Processo
**8.2.a** **Runbooks**
- Procedimentos de deploy
- Troubleshooting comum
- Procedimentos de emergência

**8.2.b** **Arquitetura**
- Diagramas de componentes
- Fluxos de dados
- Decisões arquiteturais

## 9. Regras de Compliance

### 9.1 Regulamentações
**Referência**: [10.3 Compliance e Riscos](ALL_BASE.md#103-compliance-e-riscos)

**9.1.a** **Verificação Legal**
- Verificar regulamentações locais antes do deploy
- Implementar KYC/AML se necessário
- Manter registros de auditoria

**9.1.b** **Gestão de Risco**
- Limites de exposição por ativo
- Diversificação de estratégias
- Planos de contingência

### 9.2 Auditoria
**9.2.a** **Trilha de Auditoria**
- Log de todas as decisões de trading
- Rastreabilidade completa de transações
- Backup seguro de dados críticos

**9.2.b** **Revisão de Código**
- Code review obrigatório para mudanças críticas
- Aprovação de múltiplos desenvolvedores
- Checklist de segurança

---

## Checklist de Compliance

### Antes de Cada Deploy
- [ ] Todos os testes passando
- [ ] Code review aprovado
- [ ] Documentação atualizada
- [ ] Configurações validadas
- [ ] Backup de configurações anterior
- [ ] Plano de rollback definido

### Antes de Cada Release
- [ ] Testes de performance executados
- [ ] Testes de segurança executados
- [ ] Release notes criadas
- [ ] Stakeholders notificados
- [ ] Monitoramento configurado
- [ ] Alertas testados

---

*Estas regras devem ser seguidas rigorosamente para garantir a qualidade, segurança e confiabilidade do bot de arbitragem. Violações devem ser documentadas e justificadas.*