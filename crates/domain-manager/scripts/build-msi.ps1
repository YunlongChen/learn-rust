<#
.SYNOPSIS
    构建Domain Manager的Windows MSI安装包

.DESCRIPTION
    支持使用cargo-wix或原生WiX工具链构建MSI安装包

.PARAMETER BuildType
    构建类型：debug 或 release（默认：release）

.PARAMETER Method
    构建方法：cargo-wix 或 native（默认：cargo-wix）

.PARAMETER OutputDir
    输出目录（默认：release/windows-installer）

.EXAMPLE
    .\scripts\build-msi.ps1 -BuildType release
    .\scripts\build-msi.ps1 -BuildType debug -Method native
#>

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("debug", "release")]
    [string]$BuildType = "release",
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("cargo-wix", "native")]
    [string]$Method = "cargo-wix",
    
    [Parameter(Mandatory=$false)]
    [string]$OutputDir = "release/windows-installer"
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取脚本目录和项目根目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$OutputPath = Join-Path $ProjectRoot $OutputDir

Write-Host "=== Domain Manager MSI 安装包构建 ===" -ForegroundColor Green
Write-Host "构建类型: $BuildType" -ForegroundColor Yellow
Write-Host "构建方法: $Method" -ForegroundColor Yellow
Write-Host "输出目录: $OutputPath" -ForegroundColor Yellow

# 检查cargo-wix工具
function Test-CargoWix {
    try {
        $cargoWixVersion = & cargo wix --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ cargo-wix已安装: $cargoWixVersion" -ForegroundColor Green
            return $true
        }
    } catch {
        # 忽略异常
    }
    
    Write-Host "✗ 未找到cargo-wix" -ForegroundColor Red
    Write-Host "请安装cargo-wix:" -ForegroundColor Yellow
    Write-Host "  cargo install cargo-wix" -ForegroundColor Cyan
    return $false
}

# 检查WiX工具链
function Test-WixToolset {
    try {
        $wixVersion = & candle.exe -? 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ WiX工具链已安装" -ForegroundColor Green
            return $true
        }
    } catch {
        # 忽略异常
    }
    
    Write-Host "✗ 未找到WiX工具链" -ForegroundColor Red
    Write-Host "请安装WiX Toolset v3.11+: https://wixtoolset.org/releases/" -ForegroundColor Yellow
    Write-Host "或者使用以下命令安装:" -ForegroundColor Yellow
    Write-Host "  winget install WiXToolset.WiXToolset" -ForegroundColor Cyan
    return $false
}

# 使用cargo-wix构建
function Invoke-CargoWixBuild {
    Write-Host "正在使用cargo-wix构建MSI安装包..." -ForegroundColor Yellow
    
    try {
        # 首先构建项目
        Write-Host "构建Rust项目..." -ForegroundColor Yellow
        & cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo构建失败，退出码: $LASTEXITCODE"
        }
        
        # 使用cargo-wix生成MSI
        $cargoWixArgs = @(
            "wix",
            "--package", "domain_manager",
            "--no-build",
            "--nocapture"
        )
        
        if ($BuildType -eq "debug") {
            $cargoWixArgs += "--profile", "dev"
        }
        
        & cargo @cargoWixArgs
        if ($LASTEXITCODE -ne 0) {
            throw "cargo-wix构建失败，退出码: $LASTEXITCODE"
        }
        
        # 查找生成的MSI文件
        $msiFiles = Get-ChildItem -Path "$ProjectRoot\target\wix" -Filter "*.msi" -ErrorAction SilentlyContinue
        if ($msiFiles.Count -eq 0) {
            throw "未找到生成的MSI文件"
        }
        
        $sourceMsi = $msiFiles[0].FullName
        $targetMsi = Join-Path $OutputPath "DomainManager-$BuildType-cargo-wix.msi"
        
        # 创建输出目录并复制MSI文件
        if (-not (Test-Path $OutputPath)) {
            New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
        }
        
        Copy-Item $sourceMsi $targetMsi -Force
        Write-Host "✓ MSI安装包生成成功: $targetMsi" -ForegroundColor Green
        return $targetMsi
        
    } catch {
        Write-Host "✗ cargo-wix构建失败: $($_.Exception.Message)" -ForegroundColor Red
        throw
    }
}

# 使用原生WiX构建
function Invoke-NativeWixBuild {
    Write-Host "正在使用原生WiX工具链构建MSI安装包..." -ForegroundColor Yellow
    
    # 调用原生WiX构建脚本
    $installerScript = Join-Path $ScriptDir "build-installer.ps1"
    if (-not (Test-Path $installerScript)) {
        throw "未找到原生WiX构建脚本: $installerScript"
    }
    
    try {
        & $installerScript -BuildType $BuildType -OutputDir $OutputDir
        if ($LASTEXITCODE -ne 0) {
            throw "原生WiX构建失败，退出码: $LASTEXITCODE"
        }
        
        $msiPath = Join-Path $OutputPath "DomainManager-$BuildType.msi"
        if (-not (Test-Path $msiPath)) {
            throw "未找到生成的MSI文件: $msiPath"
        }
        
        Write-Host "✓ MSI安装包生成成功: $msiPath" -ForegroundColor Green
        return $msiPath
        
    } catch {
        Write-Host "✗ 原生WiX构建失败: $($_.Exception.Message)" -ForegroundColor Red
        throw
    }
}

# 显示安装包信息
function Show-InstallerInfo {
    param([string]$MsiPath)
    
    if (Test-Path $MsiPath) {
        $msiSize = (Get-Item $MsiPath).Length / 1MB
        Write-Host "" -ForegroundColor Green
        Write-Host "=== 构建完成 ===" -ForegroundColor Green
        Write-Host "安装包路径: $MsiPath" -ForegroundColor Cyan
        Write-Host "安装包大小: $([math]::Round($msiSize, 2)) MB" -ForegroundColor Cyan
        Write-Host "构建方法: $Method" -ForegroundColor Cyan
        Write-Host "" -ForegroundColor Green
        Write-Host "安装命令:" -ForegroundColor Yellow
        Write-Host "  msiexec /i \"$MsiPath\" /quiet" -ForegroundColor Cyan
        Write-Host "卸载命令:" -ForegroundColor Yellow
        Write-Host "  msiexec /x \"$MsiPath\" /quiet" -ForegroundColor Cyan
        Write-Host "测试安装:" -ForegroundColor Yellow
        Write-Host "  msiexec /i \"$MsiPath\" /l*v install.log" -ForegroundColor Cyan
    }
}

# 主构建流程
try {
    # 根据构建方法检查环境
    if ($Method -eq "cargo-wix") {
        if (-not (Test-CargoWix)) {
            Write-Host "尝试安装cargo-wix..." -ForegroundColor Yellow
            & cargo install cargo-wix
            if ($LASTEXITCODE -ne 0) {
                Write-Host "cargo-wix安装失败，切换到原生WiX方法" -ForegroundColor Yellow
                $Method = "native"
            } else {
                Write-Host "✓ cargo-wix安装成功" -ForegroundColor Green
            }
        }
    }
    
    if ($Method -eq "native") {
        if (-not (Test-WixToolset)) {
            exit 1
        }
    }
    
    # 执行构建
    $msiPath = $null
    if ($Method -eq "cargo-wix") {
        $msiPath = Invoke-CargoWixBuild
    } else {
        $msiPath = Invoke-NativeWixBuild
    }
    
    # 显示结果
    Show-InstallerInfo -MsiPath $msiPath
    
} catch {
    Write-Host "" -ForegroundColor Red
    Write-Host "=== 构建失败 ===" -ForegroundColor Red
    Write-Host "错误信息: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "" -ForegroundColor Yellow
    Write-Host "故障排除建议:" -ForegroundColor Yellow
    Write-Host "1. 确保已安装WiX Toolset v3.11+" -ForegroundColor White
    Write-Host "2. 确保已安装cargo-wix (如果使用cargo-wix方法)" -ForegroundColor White
    Write-Host "3. 确保项目已成功构建" -ForegroundColor White
    Write-Host "4. 检查资源文件是否存在" -ForegroundColor White
    exit 1
}