# 🚀 Instalação de Dependências - Solana Arbitrage Bot

Este documento contém **todos os comandos possíveis** para instalar as dependências do projeto, independente de versão ou sistema operacional.

## 📋 Pré-requisitos

```bash
# Verificar se Rust está instalado
cargo --version

# Se não estiver instalado:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 🎯 Comandos de Instalação Automática

### **Opção 1: Scripts Automatizados (Recomendado)**

#### **Linux/macOS:**
```bash
chmod +x install-deps.sh
./install-deps.sh
```

#### **Windows:**
```powershell
PowerShell -ExecutionPolicy Bypass -File install-deps.ps1
```

### **Opção 2: Comandos Diretos Cargo**

#### **Instalação Padrão (mais simples):**
```bash
cargo build
```

#### **Se der conflito, tente em ordem:**
```bash
# 1. Atualizar dependências
cargo update && cargo build

# 2. Limpeza completa
cargo clean && rm Cargo.lock && cargo build

# 3. Forçar versões específicas
cargo update -p solana-client -p solana-sdk && cargo build

# 4. Dependências mínimas
cargo build --no-default-features
```

#### **Instalação Forçada (última opção):**
```bash
# Ignorar cache e locks
cargo build --offline --frozen || cargo build

# Reinstalação completa
rm -rf target/ Cargo.lock ~/.cargo/registry/cache/
cargo build
```

## 🔧 Comandos por Estratégia

### **Estratégia 1: Resolução Automática**
```bash
# Deixa o Cargo resolver automaticamente
cargo build
```

### **Estratégia 2: Atualização Incremental**
```bash
# Atualiza apenas dependências necessárias
cargo update
cargo build
```

### **Estratégia 3: Reset Completo**
```bash
# Remove tudo e reinstala
cargo clean
rm -f Cargo.lock
cargo build
```

### **Estratégia 4: Versões Específicas**
```bash
# Atualiza pacotes Solana específicos
cargo update -p solana-client -p solana-sdk -p anchor-lang -p anchor-spl
cargo build
```

### **Estratégia 5: Sem Features**
```bash
# Instala apenas o essencial
cargo build --no-default-features
```

### **Estratégia 6: Offline/Frozen**
```bash
# Usa cache local
cargo build --offline --frozen
```

## 🌐 Comandos por Sistema Operacional

### **Linux:**
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential pkg-config libssl-dev
cargo build

# Arch Linux
sudo pacman -S base-devel openssl pkg-config
cargo build

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel pkg-config
cargo build
```

### **macOS:**
```bash
# Com Homebrew
brew install pkg-config openssl
export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"
cargo build

# Com MacPorts
sudo port install pkgconfig openssl
cargo build
```

### **Windows:**
```powershell
# Com winget
winget install Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools

# Com chocolatey
choco install rust
choco install visualstudio2022buildtools

cargo build
```

## 🔍 Comandos de Verificação

### **Verificar Instalação:**
```bash
# Verificar se tudo está OK
cargo check

# Verificar com mais detalhes
cargo check --verbose

# Testar compilação completa
cargo build --release
```

### **Verificar Dependências:**
```bash
# Listar dependências
cargo tree

# Verificar updates disponíveis
cargo outdated  # (requer: cargo install cargo-outdated)

# Verificar conflitos
cargo tree --duplicates
```

### **Verificar Funcionalidade:**
```bash
# Executar testes
cargo test

# Executar com features específicas
cargo test --features priority-fees

# Executar testes de integração
cargo test --test integration_tests
```

## 🆘 Resolução de Problemas

### **Conflitos de Versão:**
```bash
# Ver árvore de dependências
cargo tree --duplicates

# Forçar versão específica
cargo update -p <package-name> --precise <version>

# Limpar registry
rm -rf ~/.cargo/registry/cache/
```

### **Problemas de Rede:**
```bash
# Usar mirror alternativo
export CARGO_REGISTRIES_CRATES_IO_INDEX="https://mirrors.ustc.edu.cn/crates.io-index"

# Configurar proxy
export CARGO_HTTP_PROXY=http://proxy:port

# Aumentar timeout
export CARGO_NET_GIT_FETCH_WITH_CLI=true
```

### **Problemas de SSL:**
```bash
# Linux
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# Windows
set CARGO_NET_GIT_FETCH_WITH_CLI=true

# Desabilitar verificação SSL (APENAS para teste)
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export GIT_SSL_NO_VERIFY=true
```

## 📊 Comandos de Diagnóstico

### **Informações do Sistema:**
```bash
# Versões instaladas
cargo --version
rustc --version
rustup --version

# Informações do sistema
rustc --print cfg
rustc --print target-list

# Informações detalhadas
cargo version --verbose
```

### **Debug de Compilação:**
```bash
# Compilação verbosa
cargo build --verbose

# Ver comandos executados
cargo build -vv

# Salvar log de compilação
cargo build 2>&1 | tee build.log
```

## 🎯 Comandos One-Liner

### **Instalação Completa (Linux/macOS):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source ~/.cargo/env && cargo build
```

### **Instalação com Retry (Linux/macOS):**
```bash
cargo build || (cargo update && cargo build) || (cargo clean && cargo build)
```

### **Instalação Forçada (qualquer SO):**
```bash
cargo clean && rm -f Cargo.lock && cargo update && cargo build --release
```

## ✅ Verificação Final

Após qualquer instalação, execute:

```bash
# 1. Verificar compilação
cargo check

# 2. Executar testes básicos
cargo test --lib

# 3. Compilar versão de release
cargo build --release

# 4. Verificar se pode executar
cargo run --help
```

Se todos os comandos acima funcionarem, a instalação foi bem-sucedida! 🎉

## 📞 Suporte

Se nenhum comando funcionar, verifique:

1. **Versão do Rust**: `rustc --version` (deve ser 1.70+)
2. **Conectividade**: `ping crates.io`
3. **Espaço em disco**: `df -h` (precisa de ~2GB livres)
4. **Permissões**: Execute com `sudo` se necessário

**Para mais ajuda, abra uma issue com a saída dos comandos de diagnóstico.** 