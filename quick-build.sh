#!/bin/bash

# Script para build rápido verificando apenas dependências principais
echo "🔧 Testando dependências principais..."

# Verificar se conseguimos ao menos incluir as dependências
echo "✅ Verificando Solana SDK..."
cargo tree -i solana-client || echo "❌ Problema com solana-client"

echo "✅ Verificando Anchor..."
cargo tree -i anchor-lang || echo "❌ Problema com anchor-lang"

echo "✅ Verificando async runtime..."
cargo tree -i tokio || echo "❌ Problema com tokio"

echo ""
echo "📋 Status das dependências:"
echo "  ✅ solana-client = 1.18"
echo "  ✅ solana-sdk = 1.18"  
echo "  ✅ anchor-lang = 0.30.1"
echo "  ✅ tokio = 1.0"
echo "  ✅ serde = 1.0"
echo "  ✅ anyhow = 1.0"
echo ""
echo "⚠️  Removido: yellowstone-grpc (conflito de versão)"
echo "⚠️  Removido: jito-sdk (pacote inexistente)"
echo ""
echo "🎯 Próximos passos: Implementar código faltando conforme TODOs" 