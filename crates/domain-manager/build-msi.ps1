<#
.SYNOPSIS
    构建Domain Manager的Windows MSI安装包

.DESCRIPTION
    此脚本支持使用cargo-wix和原生WiX两种方式构建Domain Manager的Windows MSI安装包。

.PARAMETER BuildType
    构建类型：debug 或 release（默认）

.PARAMETER Method
    构建方法：cargo-wix 或 wix（默认）

.PARAMETER OutputDir
    输出目录路径（默认：./dist）

.EXAMPLE
    .\build-msi.ps1
    使用默认设置构建release版本的MSI安装包

.EXAMPLE
    .\build-msi.ps1 -BuildType debug -Method wix
    使用原生WiX工具链构建debug版本的MSI安装包
#>

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("debug", "release")]
    [string]$BuildType = "release",

    [Parameter(Mandatory=$false)]
    [ValidateSet("cargo-wix", "wix")]
    [string]$Method = "wix",

    [Parameter(Mandatory=$false)]
    [string]$OutputDir = "./dist"
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取项目根目录
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

Write-Host "=== Domain Manager MSI构建脚本 ===" -ForegroundColor Cyan
Write-Host "构建类型: $BuildType" -ForegroundColor Yellow
Write-Host "构建方法: $Method" -ForegroundColor Yellow
Write-Host "输出目录: $OutputDir" -ForegroundColor Yellow
Write-Host "项目根目录: $ProjectRoot" -ForegroundColor Yellow
Write-Host ""

# 确保输出目录存在
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-Host "创建输出目录: $OutputDir" -ForegroundColor Green
}

# 使用cargo-wix构建MSI
if ($Method -eq "cargo-wix") {
    Write-Host "检查cargo-wix工具..." -ForegroundColor Yellow

    # 检查cargo-wix是否安装
    $cargoWixCheck = cargo wix --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "cargo-wix未安装，正在安装..." -ForegroundColor Yellow
        cargo install cargo-wix
        if ($LASTEXITCODE -ne 0) {
            Write-Error "cargo-wix安装失败"
            exit 1
        }
    }

    Write-Host "使用cargo-wix构建MSI安装包..." -ForegroundColor Green

    # 构建MSI
    if ($BuildType -eq "debug") {
        cargo wix --package domain_manager --no-build
    } else {
        cargo wix --package domain_manager
    }

    if ($LASTEXITCODE -eq 0) {
        # 查找生成的MSI文件
        $msiFiles = Get-ChildItem -Path "$ProjectRoot\target\wix" -Filter "*.msi" -ErrorAction SilentlyContinue
        if ($msiFiles) {
            foreach ($msi in $msiFiles) {
                $destPath = Join-Path $OutputDir $msi.Name+-+-+-+-
                Copy-Item $msi.FullName $destPath -Force
                Write-Host "MSI安装包已复制到: $destPath" -ForegroundColor Green
                Write-Host "文件大小: $([math]::Round($msi.Length / 1MB, 2)) MB" -ForegroundColor Cyan
            }
        }
    } else {
        Write-Error "cargo-wix构建失败"
        exit 1
    }
}

# 使用原生WiX工具链构建MSI
if ($Method -eq "wix") {
    Write-Host "检查WiX工具链..." -ForegroundColor Yellow

    # 检查candle.exe和light.exe
    $candleCheck = Get-Command candle.exe -ErrorAction SilentlyContinue
    $lightCheck = Get-Command light.exe -ErrorAction SilentlyContinue

    if (-not $candleCheck -or -not $lightCheck) {
        Write-Error "WiX工具链未找到，请安装WiX Toolset并确保candle.exe和light.exe在PATH中"
        exit 1
    }

    Write-Host "使用原生WiX工具链构建MSI安装包..." -ForegroundColor Green

    # 构建Rust项目
    Write-Host "构建Rust项目..." -ForegroundColor Yellow
    if ($BuildType -eq "debug") {
        cargo build --package domain_manager
    } else {
        cargo build --release --package domain_manager
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Rust项目构建失败"
        exit 1
    }

    # 确保输出目录存在
    $wixOutputDir = Join-Path $ProjectRoot "target\wix"
    if (-not (Test-Path $wixOutputDir)) {
        New-Item -ItemType Directory -Path $wixOutputDir -Force | Out-Null
    }

    # WiX源文件路径
    $wxsFile = Join-Path $PSScriptRoot "wix\domain_manager.wxs"
    if (-not (Test-Path $wxsFile)) {
        Write-Error "WiX源文件未找到: $wxsFile"
        exit 1
    }

    # 编译WiX源文件
    $wixobjFile = Join-Path $wixOutputDir "domain_manager.wixobj"
    $targetBinDir = if ($BuildType -eq "debug") { "$ProjectRoot\target\debug" } else { "$ProjectRoot\target\release" }

    Write-Host "编译WiX源文件..." -ForegroundColor Yellow
    Write-Host "目标二进制目录: $targetBinDir" -ForegroundColor Cyan
    & candle.exe -out "$wixobjFile" -dCargoTargetBinDir="$targetBinDir" -dVersion="1.0.0" "$wxsFile"

    if ($LASTEXITCODE -ne 0) {
        Write-Error "WiX源文件编译失败"
        exit 1
    }

    # 链接生成MSI
    $msiFile = Join-Path $wixOutputDir "domain_manager.msi"
    Write-Host "链接生成MSI安装包..." -ForegroundColor Yellow
    & light.exe -out $msiFile -ext WixUIExtension $wixobjFile

    if ($LASTEXITCODE -eq 0) {
        # 复制MSI到输出目录
        $destPath = Join-Path $OutputDir "domain_manager.msi"
        Copy-Item $msiFile $destPath -Force

        $msiInfo = Get-Item $destPath
        Write-Host "MSI安装包已生成: $destPath" -ForegroundColor Green
        Write-Host "文件大小: $([math]::Round($msiInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan
    } else {
        Write-Error "MSI链接失败"
        exit 1
    }
}

Write-Host ""
Write-Host "=== 构建完成 ===" -ForegroundColor Green
Write-Host "安装包位置: $OutputDir" -ForegroundColor Cyan

# 显示安装包信息
$msiFiles = Get-ChildItem -Path $OutputDir -Filter "*.msi"
if ($msiFiles) {
    Write-Host ""
    Write-Host "生成的安装包:" -ForegroundColor Yellow
    foreach ($msi in $msiFiles) {
        Write-Host "  - $($msi.Name) ($([math]::Round($msi.Length / 1MB, 2)) MB)" -ForegroundColor White
    }
}
