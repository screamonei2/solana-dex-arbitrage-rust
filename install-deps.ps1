# Script PowerShell para instalar dependencias do Solana Arbitrage Bot
# Funciona independente de versao especifica - Windows

Write-Host "Instalando dependencias do Solana Arbitrage Bot..." -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Green

# Funcao para tentar diferentes estrategias
function Try-Install {
    param(
        [string]$Strategy,
        [string]$Description
    )
    
    Write-Host ""
    Write-Host "Tentativa: $Description" -ForegroundColor Cyan
    Write-Host "-----------------------------------" -ForegroundColor Cyan
    
    try {
        switch ($Strategy) {
            "auto" {
                Write-Host "Instalacao automatica com resolucao de conflitos..." -ForegroundColor Yellow
                $result = cargo build
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "Sucesso com instalacao automatica!" -ForegroundColor Green
                    return $true
                }
                throw "Cargo build falhou"
            }
            "update" {
                Write-Host "Atualizando dependencias..." -ForegroundColor Yellow
                cargo update
                cargo build
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "Sucesso com atualizacao!" -ForegroundColor Green
                    return $true
                }
                throw "Update falhou"
            }
            "force" {
                Write-Host "Instalacao forcada com override..." -ForegroundColor Yellow
                cargo build --offline --frozen 2>$null
                if ($LASTEXITCODE -ne 0) {
                    cargo build
                }
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "Sucesso com instalacao forcada!" -ForegroundColor Green
                    return $true
                }
                throw "Instalacao forcada falhou"
            }
            "clean" {
                Write-Host "Limpando cache e reinstalando..." -ForegroundColor Yellow
                cargo clean
                Remove-Item -Path "Cargo.lock" -ErrorAction SilentlyContinue
                cargo build
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "Sucesso com limpeza!" -ForegroundColor Green
                    return $true
                }
                throw "Limpeza falhou"
            }
            "minimal" {
                Write-Host "Instalacao minimal..." -ForegroundColor Yellow
                cargo check
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "Sucesso com instalacao minimal!" -ForegroundColor Green
                    return $true
                }
                throw "Instalacao minimal falhou"
            }
        }
    }
    catch {
        Write-Host "Falhou: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Lista de estrategias para tentar
$strategies = @(
    @{name="auto"; desc="Instalacao Automatica"},
    @{name="update"; desc="Atualizacao de Dependencias"},
    @{name="force"; desc="Instalacao Forcada"},
    @{name="clean"; desc="Limpeza e Reinstalacao"},
    @{name="minimal"; desc="Instalacao Minimal"}
)

Write-Host ""
Write-Host "Verificando Rust..." -ForegroundColor Blue
try {
    $rustVersion = cargo --version
    Write-Host "Rust encontrado: $rustVersion" -ForegroundColor Green
}
catch {
    Write-Host "ERRO: Rust nao encontrado!" -ForegroundColor Red
    Write-Host "Instale Rust primeiro: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Tentar cada estrategia ate uma funcionar
foreach ($strategy in $strategies) {
    if (Try-Install -Strategy $strategy.name -Description $strategy.desc) {
        Write-Host ""
        Write-Host "SUCESSO! Dependencias instaladas com: $($strategy.desc)" -ForegroundColor Green
        Write-Host ""
        Write-Host "Proximos passos:" -ForegroundColor Blue
        Write-Host "  cargo run       - Executar o bot" -ForegroundColor White
        Write-Host "  cargo test      - Executar testes" -ForegroundColor White
        Write-Host "  cargo check     - Verificar codigo" -ForegroundColor White
        exit 0
    }
}

# Se chegou aqui, todas as tentativas falharam
Write-Host ""
Write-Host "ERRO: Todas as tentativas de instalacao falharam!" -ForegroundColor Red
Write-Host ""
Write-Host "Tente manualmente:" -ForegroundColor Yellow
Write-Host "  cargo clean" -ForegroundColor White
Write-Host "  del Cargo.lock" -ForegroundColor White
Write-Host "  cargo build" -ForegroundColor White
Write-Host ""
Write-Host "Ou consulte INSTALL.md para mais opcoes" -ForegroundColor Yellow
exit 1 