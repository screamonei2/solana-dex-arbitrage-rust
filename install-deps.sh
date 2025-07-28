#!/bin/bash

# Script para instalar dependências do Solana Arbitrage Bot
# Funciona independente de versão específica

echo "🚀 Instalando dependências do Solana Arbitrage Bot..."
echo "=================================================="

# Função para tentar diferentes estratégias
try_install() {
    local strategy=$1
    local description=$2
    
    echo ""
    echo "📦 Tentativa: $description"
    echo "-----------------------------------"
    
    case $strategy in
        "auto")
            echo "🔄 Instalação automática com resolução de conflitos..."
            cargo install --locked || cargo install || return 1
            ;;
        "update")
            echo "🔄 Atualizando dependências para versões mais recentes..."
            cargo update && cargo build || return 1
            ;;
        "clean")
            echo "🧹 Limpando cache e reinstalando..."
            cargo clean
            rm -f Cargo.lock
            cargo build || return 1
            ;;
        "minimal")
            echo "🎯 Instalação com dependências mínimas..."
            cargo build --no-default-features || return 1
            ;;
        "latest")
            echo "🆕 Forçando versões mais recentes..."
            cargo update -p solana-client -p solana-sdk -p anchor-lang -p anchor-spl
            cargo build || return 1
            ;;
    esac
    
    return 0
}

# Verificar se Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo não encontrado. Instalando..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Verificar versão do Rust
echo "🦀 Versão do Rust:"
cargo --version
rustc --version

# Estratégias de instalação em ordem de preferência
strategies=(
    "auto:Instalação automática padrão"
    "update:Atualizar dependências existentes"
    "clean:Limpeza completa e reinstalação"
    "latest:Forçar versões mais recentes"
    "minimal:Apenas dependências essenciais"
)

success=false

for strategy_info in "${strategies[@]}"; do
    IFS=':' read -r strategy description <<< "$strategy_info"
    
    if try_install "$strategy" "$description"; then
        echo "✅ Sucesso com estratégia: $description"
        success=true
        break
    else
        echo "❌ Falhou com estratégia: $description"
        continue
    fi
done

if [ "$success" = true ]; then
    echo ""
    echo "🎉 INSTALAÇÃO CONCLUÍDA COM SUCESSO!"
    echo "=================================================="
    echo "✅ Todas as dependências foram instaladas"
    echo "✅ Projeto pronto para desenvolvimento"
    echo ""
    echo "📋 Próximos passos:"
    echo "   cargo build        # Compilar o projeto"
    echo "   cargo test         # Executar testes"
    echo "   cargo run          # Executar o bot"
    echo ""
    
    # Verificar se compilação está funcionando
    echo "🧪 Testando compilação..."
    if cargo check; then
        echo "✅ Compilação OK!"
    else
        echo "⚠️  Compilação com avisos - verifique o código"
    fi
    
else
    echo ""
    echo "❌ FALHA NA INSTALAÇÃO"
    echo "=================================================="
    echo "Nenhuma estratégia de instalação funcionou."
    echo ""
    echo "🔧 Soluções manuais:"
    echo "1. Verificar conectividade com internet"
    echo "2. Atualizar Rust: rustup update"
    echo "3. Verificar se todas as dependências existem"
    echo "4. Instalar manualmente: cargo add <dependencia>"
    echo ""
    exit 1
fi 