Perfeito. Vou reunir toda a documentação técnica, APIs, contratos, padrões e práticas recomendadas das DEXs na rede Solana que atualmente possuem o par BONK/SOL, incluindo agregadores como Jupiter. Isso incluirá detalhes sobre swaps, taxas, segurança, atomicidade e possibilidades como arbitragem triangular.

Vou organizar tudo em um arquivo chamado **BASE.md**, que servirá como guia oficial e completo para o seu projeto de bot de arbitragem em Rust. Assim que estiver pronto, volto com o material estruturado para sua revisão.


# BASE.md – Guia Completo de DEXs Solana (par BONK/SOL)

Este documento reúne informações oficiais das principais DEXs (trocas descentralizadas) na rede Solana que oferecem o par de tokens **BONK/SOL**. Nele você encontrará descrições, endpoints de API, padrões de código e melhores práticas para criar um bot de arbitragem robusto, atômico e seguro em Rust. Todas as referências a seguir provêm das documentações oficiais destas plataformas e guias reconhecidos.

## Raydium

* **O que é:** Raydium é um AMM integrado ao livro de ordens Serum, permitindo swaps rápidos de tokens Solana com alta liquidez. Ele fornece duas APIs principais: a “Trade API” para swaps diretos e um SDK TypeScript para desenvolvedores.
* **Documentação e API:** A *Trade API* oficial do Raydium executa swaps em apenas duas chamadas (citação de rota e envio de transação). Por exemplo, o guia de Trade API explica parâmetros de requisição `inputMint`, `outputMint`, `amount`, `slippageBps`, além de detalhes de transação (versão, wrap/unwrap SOL, contas ATA, etc). O endpoint principal é `https://api-v3.raydium.io/` (docs em `/docs`), conforme mostra a documentação.
* **Integração em Rust:** Não há SDK Rust oficial público, mas existe o crate [`solana_router_raydium`](https://docs.rs/solana-router-raydium) que implementa instruções para interagir com os pools Raydium. Por exemplo, este crate fornece métodos como `swap` para “trocar tokens em um pool Raydium”, além de adicionar/remover liquidez e stake de LP. Para código Rust, você pode usar o `solana-sdk` ou o cliente Anchor para montar transações com estas instruções. Em TypeScript, o pacote `@raydium-io/raydium-sdk-v2` também permite gerar transações de swap.
* **Fluxo de Swap:** Em geral, o processo envolve obter uma cotação via API REST (GET ou POST) e então assinar/ enviar a transação de swap. Um exemplo de uso (não oficial) pode seguir a ordem: solicitar rota na API Trade (com mints, quantia, slippage), usar o JSON retornado para montar instruções de swap, assinar e enviar pela RPC de Solana. As taxas típicas de swap em pools AMM (por exemplo 0.25%) devem ser consideradas ao definir slippage.
* **Referências:** Descrição de Raydium como DEX AMM integrado ao Serum; Trade API oficial e parâmetros de swap; crate Rust de integração.

## Orca (Whirlpools)

* **O que é:** Orca é uma DEX concentrada (CLMM) líder em Solana, destacada por confiança da comunidade. Ela opera pools de liquidez concentrada (“Whirlpools”) com maior eficiência de capital.
* **Documentação e API:** Orca mantém documentação em GitBook e no GitHub. A visão geral oficial descreve Orca como *“o DEX mais confiável em Solana e Eclipse, com um modelo único de AMM de Liquidez Concentrada”*. Seus guias técnicos detalham o programa Whirlpool (ID de programa publicado no GitHub) e a estrutura de contas usada nos swaps.
* **SDKs e Integração em Rust:** Para desenvolvedores, Orca oferece SDKs de alto nível. Por exemplo, o [Rust SDK `orca_whirlpools`](https://crates.io/crates/orca_whirlpools) é a camada recomendada, abstraindo as complexidades dos Whirlpools. Ele permite carregar pools, calcular cotações e emitir instruções de swap. Há também o crate [`solana_router_orca`](https://docs.rs/solana-router-orca) (do Solana Farm SDK) que fornece instruções prontas para Orca, incluindo `swap` para pools Orca. Em TypeScript, usa-se `@orca-so/whirlpools-sdk` e `@orca-so/common-sdk`.
* **Fluxo de Swap:** Usando o SDK ou o cliente Anchor, cria-se um contexto com `WhirlpoolContext.withProvider(provider, ORCA_WHIRLPOOL_PROGRAM_ID)`, carrega-se o Whirlpool desejado e então gera-se uma cotação (por input ou output) e executa-se o swap on-chain. Um exemplo em código (Node/Anchor) requer as variáveis de ambiente `ANCHOR_PROVIDER_URL` e `ANCHOR_WALLET` definidas (RPC e chave privada). A transação resultante inclui a instrução de swap do programa Orca, que assegura mudança atômica de preços.
* **Best Practices:** Como em qualquer integração Solana, siga boas práticas: armazene RPC e chaves em `.env` ignorado no Git (using crates como `dotenv`) e trate erros de transação.
* **Referências:** Visão geral Orca com CLMM; SDKs Orca (Rust e TS); exemplo de integração (requisição de variáveis ambiente e uso de `buildWhirlpoolClient`); crate Rust de swap Orca.

## Aldrin

* **O que é:** Aldrin é outra DEX/AMM no Solana (lançada em 2021) que oferece swaps spot, futuros e pools de liquidez. É conhecido pelo token RIN.
* **Documentação e API:** A documentação oficial (GitHub) descreve o uso da *Aldrin SDK* (TypeScript) para operações de swap e liquidez. Não há publicamente uma documentação de API REST aberta, mas a SDK faz as chamadas necessárias via transações.
* **SDKs e Integração em Rust:** Aldrin oferece uma SDK oficial (TypeScript) detalhada em seu repositório. Por exemplo, para swap: instancie `TokenSwap.initialize()`, busque preço com `getPrice({mintFrom, mintTo})` e então execute `swap({ wallet, minIncomeAmount, mintFrom, mintTo })`. Para Rust, existe um crate [`aldrin_swap_calc`](https://docs.rs/aldrin_swap_calc) para cálculos de swap, usado internamente pela SDK. Além disso, o SDK relata instruções de liquidez e farming.
* **Fluxo de Swap:** O uso típico é parecido: inicializa-se o cliente (via Anchor ou SDK), consulta-se preço e depois envia-se a transação de swap. As LPs na Aldrin seguem o modelo constante-produto. Em Rust, seria necessário usar `anchor-client` ou o crate de cálculo para construir as instruções.
* **Referências:** Descrição de Aldrin como DEX Solana com AMM e futuros; exemplo de uso da SDK (getPrice + swap); biblioteca Rust para cálculo de swap.

## Lifinity

* **O que é:** Lifinity é um “market maker proativo” que diferencia-se por usar oráculos de preço para reduzir perdas impermanentes. Seus pools ajustam ativos baseados em sinais externos, revertendo impermanent loss em lucro para os provedores.
* **Documentação e API:** A documentação (GitBook) explica que o mecanismo central é um oráculo interno atualizado frequentemente. Não há API pública de swap, pois funciona integrado no protocolo. Para interagir, deve-se seguir instruções de contrato no on-chain.
* **Integração:** Não existe SDK público no momento. Para incluir Lifinity, o bot teria que interagir diretamente com o programa deles. A documentação sugere que o preço é determinado pelo oráculo antes de cada swap, então provavelmente os swap logic/txns são padrão e podem ser construídos via `anchor-client`, apontando para o ID de programa de Lifinity. (Em pesquisa, *não* encontramos SDKs oficiais, o que implica uso de `solana-program` ou instruções CPI).
* **Referências:** Descrição de Lifinity como AMM proativo com oráculo que “elimina dependência de arbitrageurs”; design do protocolo para eficiência de capital.

## Jupiter (Agregador)

* **O que é:** Jupiter é um *agregador* de liquidez que conecta múltiplas DEXs Solana (Orca, Raydium, Aldrin etc.) para encontrar as melhores rotas de swap. Ele divide ordens entre AMMs para otimizar preço e suporta swaps entre qualquer par de tokens.
* **Documentação e API:** A documentação do desenvolvedor ([https://dev.jup.ag](https://dev.jup.ag)) lista várias APIs: Ultra API, Swap API, Trigger API, Token API, Price API etc. Em especial, o **Swap API** permite que você peça cotação (“quote”), monte a transação e depois a envie. O motor de roteamento (“Metis”) agrega liquidez em todas DEX integradas. A Jupiter cobra *zero taxas de plataforma* sobre swaps (apenas a taxa normal de rede e AMM).
* **Ultra API:** Para integração simplificada, recomenda-se a **Ultra API**, que automatiza muitas responsabilidades do desenvolvedor. Conforme a documentação, ela lida automaticamente com RPC, seleção de taxas (priority fee, Jito etc.), slippage ótimo, transmissão e parsing de transações. Assim, o dev só precisa requisitar a swap e fornecer endpoint/fundo.
* **Fluxo de Swap:** Com a Jupiter Swap API tradicional (quando mais controle é necessário), o fluxo é: 1) chamar `/quote` com os mints e quantias, 2) receber rotas e instruções, 3) mandar `/swap` para construir TX, 4) assinar/enviar transação usando um RPC de Solana. O Ultra API faz parte desse fluxo reduzindo a complexidade.
* **Integração:** Como Jupiter é baseado em REST/JSON, você pode chamá-lo de Rust usando uma biblioteca HTTP (por exemplo `reqwest`). Não há SDK Rust oficial, mas a chamada é simples: por exemplo, `https://quote-api.jup.ag/v1/quote?inputMint=...&outputMint=...&amount=...&slippageBps=...`. Alternativamente, use `jupiter-swap` no npm se estiver usando JS/TS.
* **Referências:** Visão geral do Swap API Jupiter; lista de APIs disponíveis; recomendação do Ultra API para simplificar (autogerenciamento de RPC/fees); processo de cotação e swap Jupiter.

## OpenBook (Serum)

* **O que é:** OpenBook é o DEX de livro de ordens (CLOB) da comunidade Serum em Solana. É uma sequência do Serum original e permite criar/usar mercados limitados para pares de tokens.
* **Mercado BONK/SOL:** O par BONK/SOL existe como um mercado em OpenBook (endereço de mercado **Hs97...9CF** no devnet/mainnet). Isso significa que há ordens de compra/venda limitadas para BONK em SOL.
* **Integração:** Para usar, o bot precisaria interagir com o programa OpenBook. É possível usar o cliente Anchor ou `@project-serum/serum` (JavaScript) para acessar o livro de ordens. Em Rust, você pode usar o crate `serum-dex` ou o `solana-client` para consultar `getOrderbook`, `placeOrder`, `settleFunds` etc. Certifique-se de criar contas de ordens (PDA) e gerenciar longitudes (book depth) conforme a especificação do programa OpenBook.
* **Referências:** Exemplo de configuração no repo de *openbook-candles*, que lista o mercado BONK/SOL com seu address.

## Boas Práticas de Desenvolvimento

* **Variáveis de Ambiente:** Separe credenciais (chaves privadas, RPC URLs) do código fonte. Use arquivos `.env` e o crate `dotenv` em Rust para carregar variáveis, e coloque o `.env` no `.gitignore`. Por exemplo, defina `SOLANA_RPC_URL` e sua chave secreta no `.env` e carregue no código. Isso evita vazamento de segredos ao versionar código.
* **Transações Atômicas:** Execute cada arbitragem em uma única transação (múltiplas instruções). Assim, todas as etapas (compra/venda) são atômicas – se alguma falhar por falta de liquidez, toda a transação reverte, evitando perdas parciais. Em Solana, basta incluir todas as instruções de swap de diferentes DEXs em uma transação só e assiná-la. Se for usar Jupiter, ele já cria um único TX completo.
* **Estratégias de Arbitragem:** Implemente *arbitragem direta* (trocar BONK↔SOL em DEXs diferentes) e *triangular* (ex: SOL→BONK→USDC→SOL) para aproveitar discrepâncias. Uma abordagem é buscar preços de compra/venda em todos os DEXs (via suas APIs ou on-chain) e calcular rotas potenciais. O agregador Jupiter pode ser usado para simular rotas, ou você pode cruzar dados de pools manualmente. Lembre-se de incluir as taxas de swap ao calcular lucros.
* **Bibliotecas e Ferramentas:** Em Rust, use `solana-client` e `anchor-client` para comunicação RPC; crates dos DEX (por exemplo `solana_router_orca`, `solana_router_raydium`); e bibliotecas auxiliares (`spl-token`, `spl-associated-token-account`). Siga padrões de codificação idiomáticos (por exemplo, `anyhow`/`thiserror` para erros). Habilite a função de taxa prioritária (compute unit price) nas transações, conforme recomendado pelo Raydium. Teste em Devnet antes de Mainnet.
* **Controle de Versionamento:** Use `.gitignore` para não incluir chaves nem logs sensíveis no repositório. Mantenha a biblioteca e o código organizados: por exemplo, um módulo de “rotas” para cada DEX, um de cálculo de oportunidades de arbitragem etc.
* **Segurança:** Sempre simule transações antes (com `simulateTransaction`) para evitar falhas custosas em Mainnet. Valide sempre retornos de API e trate casos de erro (slippage, liquidez insuficiente).

## Resumo

Este manual consolida os detalhes técnicos das principais DEXs Solana que possuem pool BONK/SOL, incluindo suas APIs, SDKs e padrões de integração. Ao desenvolver o bot de arbitragem em Rust, utilize as bibliotecas oficiais indicadas e siga as práticas acima (variáveis de ambiente, atomicidade de transação, tratamento de erros). O uso de agregadores como o Jupiter simplifica a busca de melhores rotas, mas você pode sempre interagir diretamente com os programas (Whirlpools Orca, Raydium AMM, OpenBook) conforme demonstrado nas documentações referenciadas.

**Fontes:** Documentações oficiais das DEXs e guias de integração (entre outras referenciadas acima).
