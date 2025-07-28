# Makefile para Solana Arbitrage Bot
# Comandos de instalação independentes de versão

.PHONY: help install install-auto install-force install-clean install-update install-minimal check build test run clean setup

# Configurações
RUST_VERSION_MIN := 1.70.0
CARGO := cargo

# Cores para output
RED := \033[31m
GREEN := \033[32m
YELLOW := \033[33m
BLUE := \033[34m
RESET := \033[0m

help: ## 📋 Mostrar ajuda com todos os comandos disponíveis
	@echo "$(GREEN)🚀 Solana Arbitrage Bot - Comandos de Instalação$(RESET)"
	@echo "$(GREEN)================================================$(RESET)"
	@echo ""
	@echo "$(BLUE)📦 Comandos de Instalação:$(RESET)"
	@echo "  make install        - Instalação automática (padrão)"
	@echo "  make install-auto   - Tentativa automática com múltiplas estratégias"
	@echo "  make install-force  - Instalação forçada (limpa tudo)"
	@echo "  make install-clean  - Limpeza completa e reinstalação"
	@echo "  make install-update - Atualizar dependências existentes"
	@echo "  make install-minimal- Instalação apenas com dependências essenciais"
	@echo ""
	@echo "$(BLUE)🔍 Comandos de Verificação:$(RESET)"
	@echo "  make check          - Verificar compilação"
	@echo "  make build          - Compilar projeto"
	@echo "  make test           - Executar testes"
	@echo "  make run            - Executar o bot"
	@echo ""
	@echo "$(BLUE)🧹 Comandos de Limpeza:$(RESET)"
	@echo "  make clean          - Limpar arquivos de build"
	@echo "  make clean-all      - Limpeza completa (cache + builds)"
	@echo ""
	@echo "$(BLUE)⚙️  Comandos de Setup:$(RESET)"
	@echo "  make setup          - Setup completo do ambiente"
	@echo "  make deps-info      - Informações sobre dependências"
	@echo "  make rust-update    - Atualizar Rust"
	@echo ""

setup: ## ⚙️ Setup completo do ambiente de desenvolvimento
	@echo "$(YELLOW)🔧 Configurando ambiente de desenvolvimento...$(RESET)"
	@which rustc > /dev/null || { echo "$(RED)❌ Rust não encontrado. Instalando...$(RESET)"; curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; }
	@rustup update
	@rustup component add clippy rustfmt
	@echo "$(GREEN)✅ Setup concluído!$(RESET)"

install: install-auto ## 📦 Instalação padrão (alias para install-auto)

install-auto: check-rust ## 📦 Instalação automática com múltiplas estratégias
	@echo "$(YELLOW)🚀 Tentando instalação automática...$(RESET)"
	@$(CARGO) build || { \
		echo "$(YELLOW)🔄 Primeira tentativa falhou, atualizando dependências...$(RESET)"; \
		$(CARGO) update && $(CARGO) build; \
	} || { \
		echo "$(YELLOW)🧹 Segunda tentativa falhou, limpando e tentando novamente...$(RESET)"; \
		$(CARGO) clean && rm -f Cargo.lock && $(CARGO) build; \
	} || { \
		echo "$(YELLOW)💪 Tentativa final com versões específicas...$(RESET)"; \
		$(CARGO) update -p solana-client -p solana-sdk -p anchor-lang -p anchor-spl && $(CARGO) build; \
	}
	@echo "$(GREEN)✅ Instalação automática concluída!$(RESET)"

install-force: check-rust ## 💪 Instalação forçada (remove tudo e reinstala)
	@echo "$(YELLOW)💥 Instalação forçada (limpeza completa)...$(RESET)"
	@$(CARGO) clean
	@rm -f Cargo.lock
	@rm -rf target/
	@rm -rf ~/.cargo/registry/cache/ || true
	@$(CARGO) update
	@$(CARGO) build --release
	@echo "$(GREEN)✅ Instalação forçada concluída!$(RESET)"

install-clean: check-rust ## 🧹 Limpeza e reinstalação
	@echo "$(YELLOW)🧹 Limpando e reinstalando...$(RESET)"
	@$(CARGO) clean
	@rm -f Cargo.lock
	@$(CARGO) build
	@echo "$(GREEN)✅ Limpeza e reinstalação concluída!$(RESET)"

install-update: check-rust ## 🔄 Atualizar dependências existentes
	@echo "$(YELLOW)🔄 Atualizando dependências...$(RESET)"
	@$(CARGO) update
	@$(CARGO) build
	@echo "$(GREEN)✅ Atualização concluída!$(RESET)"

install-minimal: check-rust ## 🎯 Instalação com dependências mínimas
	@echo "$(YELLOW)🎯 Instalação com dependências mínimas...$(RESET)"
	@$(CARGO) build --no-default-features
	@echo "$(GREEN)✅ Instalação mínima concluída!$(RESET)"

check: ## 🔍 Verificar se o código compila
	@echo "$(BLUE)🔍 Verificando compilação...$(RESET)"
	@$(CARGO) check
	@echo "$(GREEN)✅ Verificação concluída!$(RESET)"

build: ## 🏗️ Compilar o projeto
	@echo "$(BLUE)🏗️ Compilando projeto...$(RESET)"
	@$(CARGO) build --release
	@echo "$(GREEN)✅ Compilação concluída!$(RESET)"

test: ## 🧪 Executar testes
	@echo "$(BLUE)🧪 Executando testes...$(RESET)"
	@$(CARGO) test
	@echo "$(GREEN)✅ Testes concluídos!$(RESET)"

run: ## 🚀 Executar o bot de arbitragem
	@echo "$(BLUE)🚀 Executando bot de arbitragem...$(RESET)"
	@$(CARGO) run

clean: ## 🧹 Limpar arquivos de build
	@echo "$(YELLOW)🧹 Limpando arquivos de build...$(RESET)"
	@$(CARGO) clean
	@echo "$(GREEN)✅ Limpeza concluída!$(RESET)"

clean-all: ## 🗑️ Limpeza completa (cache + builds)
	@echo "$(YELLOW)🗑️ Limpeza completa...$(RESET)"
	@$(CARGO) clean
	@rm -f Cargo.lock
	@rm -rf target/
	@rm -rf ~/.cargo/registry/cache/ || true
	@echo "$(GREEN)✅ Limpeza completa concluída!$(RESET)"

deps-info: ## 📊 Informações sobre dependências
	@echo "$(BLUE)📊 Informações sobre dependências:$(RESET)"
	@echo ""
	@echo "$(YELLOW)🦀 Versão do Rust:$(RESET)"
	@rustc --version
	@cargo --version
	@echo ""
	@echo "$(YELLOW)📦 Árvore de dependências:$(RESET)"
	@$(CARGO) tree || echo "Execute 'cargo install cargo-tree' para ver a árvore"
	@echo ""
	@echo "$(YELLOW)🔍 Dependências duplicadas:$(RESET)"
	@$(CARGO) tree --duplicates || echo "Nenhuma duplicata encontrada"

rust-update: ## 🔄 Atualizar Rust para a versão mais recente
	@echo "$(YELLOW)🔄 Atualizando Rust...$(RESET)"
	@rustup update
	@rustup component add clippy rustfmt
	@echo "$(GREEN)✅ Rust atualizado!$(RESET)"

check-rust: ## ✅ Verificar se Rust está instalado
	@which rustc > /dev/null || { \
		echo "$(RED)❌ Rust não encontrado!$(RESET)"; \
		echo "$(YELLOW)Instale com: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh$(RESET)"; \
		exit 1; \
	}

# Comandos especiais para diferentes situações
install-windows: ## 🪟 Instalação específica para Windows
	@echo "$(YELLOW)🪟 Instalação para Windows...$(RESET)"
	@powershell -ExecutionPolicy Bypass -File install-deps.ps1

install-linux: ## 🐧 Instalação específica para Linux
	@echo "$(YELLOW)🐧 Instalação para Linux...$(RESET)"
	@chmod +x install-deps.sh
	@./install-deps.sh

install-macos: install-linux ## 🍎 Instalação específica para macOS (igual ao Linux)

# Meta-comandos úteis
all: install build test ## 🎯 Fazer tudo: instalar, compilar e testar

dev: install check ## 👨‍💻 Setup para desenvolvimento

ci: install-minimal build test ## 🤖 Pipeline de CI/CD

# Comando para troubleshooting
debug: ## 🐛 Debug de instalação
	@echo "$(BLUE)🐛 Informações de debug:$(RESET)"
	@echo ""
	@echo "$(YELLOW)Sistema:$(RESET)"
	@uname -a || systeminfo 2>/dev/null || echo "Sistema desconhecido"
	@echo ""
	@echo "$(YELLOW)Rust:$(RESET)"
	@rustc --version || echo "Rust não instalado"
	@cargo --version || echo "Cargo não disponível"
	@echo ""
	@echo "$(YELLOW)Variáveis de ambiente Cargo:$(RESET)"
	@env | grep CARGO || echo "Nenhuma variável CARGO encontrada"
	@echo ""
	@echo "$(YELLOW)Espaço em disco:$(RESET)"
	@df -h . || dir || echo "Não foi possível verificar espaço"
	@echo ""
	@echo "$(YELLOW)Conectividade:$(RESET)"
	@ping -c 1 crates.io > /dev/null 2>&1 && echo "✅ Conectividade OK" || echo "❌ Problemas de conectividade"

# Comando para mostrar status
status: ## 📊 Status atual do projeto
	@echo "$(BLUE)📊 Status do projeto:$(RESET)"
	@echo ""
	@if [ -f "Cargo.lock" ]; then \
		echo "$(GREEN)✅ Cargo.lock existe$(RESET)"; \
	else \
		echo "$(YELLOW)⚠️  Cargo.lock não encontrado$(RESET)"; \
	fi
	@if [ -d "target" ]; then \
		echo "$(GREEN)✅ Diretório target existe$(RESET)"; \
	else \
		echo "$(YELLOW)⚠️  Diretório target não encontrado$(RESET)"; \
	fi
	@echo ""
	@$(CARGO) check --quiet && echo "$(GREEN)✅ Compilação OK$(RESET)" || echo "$(RED)❌ Problemas de compilação$(RESET)" 