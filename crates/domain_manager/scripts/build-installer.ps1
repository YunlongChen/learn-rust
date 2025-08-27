<#
.SYNOPSIS
    构建Domain Manager的Windows MSI安装包

.DESCRIPTION
    使用WiX工具链编译生成Windows安装包，支持debug和release模式

.PARAMETER BuildType
    构建类型：debug 或 release（默认：release）

.PARAMETER OutputDir
    输出目录（默认：release/windows-installer）

.EXAMPLE
    .\scripts\build-installer.ps1 -BuildType release
    .\scripts\build-installer.ps1 -BuildType debug -OutputDir "custom/path"
#>

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("debug", "release")]
    [string]$BuildType = "release",
    
    [Parameter(Mandatory=$false)]
    [string]$OutputDir = "release/windows-installer"
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取脚本目录和项目根目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$ResourcePath = Join-Path $ProjectRoot "resources"
$WixSourcePath = Join-Path $ResourcePath "packaging\windows\setup.wxs"
$BinaryPath = "..\..\target\$BuildType"
$OutputPath = Join-Path $ProjectRoot $OutputDir

Write-Host "=== Domain Manager Windows 安装包构建 ===" -ForegroundColor Green
Write-Host "构建类型: $BuildType" -ForegroundColor Yellow
Write-Host "输出目录: $OutputPath" -ForegroundColor Yellow

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

# 检查必要文件
function Test-RequiredFiles {
    $requiredFiles = @(
        $WixSourcePath,
        "$ProjectRoot\..\..\target\$BuildType\domain_manager.exe"
    )
    
    $missing = @()
    foreach ($file in $requiredFiles) {
        if (-not (Test-Path $file)) {
            $missing += $file
        }
    }
    
    if ($missing.Count -gt 0) {
        Write-Host "✗ 缺少必要文件:" -ForegroundColor Red
        foreach ($file in $missing) {
            Write-Host "  - $file" -ForegroundColor Red
        }
        return $false
    }
    
    Write-Host "✓ 所有必要文件存在" -ForegroundColor Green
    return $true
}

# 创建输出目录
function New-OutputDirectory {
    if (-not (Test-Path $OutputPath)) {
        New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
        Write-Host "✓ 创建输出目录: $OutputPath" -ForegroundColor Green
    }
}

# 编译WiX源文件
function Invoke-WixCompile {
    Write-Host "正在编译WiX源文件..." -ForegroundColor Yellow
    
    $wixObjPath = Join-Path $OutputPath "setup.wixobj"
    $candleArgs = @(
        "-out", $wixObjPath,
        "-dBinaryPath=$BinaryPath",
        "-dResourcePath=$ResourcePath",
        $WixSourcePath
    )
    
    try {
        & candle.exe @candleArgs
        if ($LASTEXITCODE -ne 0) {
            throw "Candle编译失败，退出码: $LASTEXITCODE"
        }
        Write-Host "✓ WiX源文件编译成功" -ForegroundColor Green
        return $wixObjPath
    } catch {
        Write-Host "✗ WiX编译失败: $($_.Exception.Message)" -ForegroundColor Red
        throw
    }
}

# 链接生成MSI
function Invoke-WixLink {
    param([string]$WixObjPath)
    
    Write-Host "正在生成MSI安装包..." -ForegroundColor Yellow
    
    $msiPath = Join-Path $OutputPath "DomainManager-$BuildType.msi"
    $lightArgs = @(
        "-out", $msiPath,
        $WixObjPath
    )
    
    try {
        & light.exe @lightArgs
        if ($LASTEXITCODE -ne 0) {
            throw "Light链接失败，退出码: $LASTEXITCODE"
        }
        Write-Host "✓ MSI安装包生成成功: $msiPath" -ForegroundColor Green
        return $msiPath
    } catch {
        Write-Host "✗ MSI生成失败: $($_.Exception.Message)" -ForegroundColor Red
        throw
    }
}

# 主构建流程
try {
    # 检查环境
    if (-not (Test-WixToolset)) {
        exit 1
    }
    
    if (-not (Test-RequiredFiles)) {
        Write-Host "请先运行构建脚本生成可执行文件:" -ForegroundColor Yellow
        Write-Host "  .\scripts\build.ps1 -BuildType $BuildType" -ForegroundColor Cyan
        exit 1
    }
    
    # 创建输出目录
    New-OutputDirectory
    
    # 编译和链接
    $wixObjPath = Invoke-WixCompile
    $msiPath = Invoke-WixLink -WixObjPath $wixObjPath
    
    # 清理临时文件
    Remove-Item $wixObjPath -Force -ErrorAction SilentlyContinue
    
    # 显示结果
    $msiSize = (Get-Item $msiPath).Length / 1MB
    Write-Host "" -ForegroundColor Green
    Write-Host "=== 构建完成 ===" -ForegroundColor Green
    Write-Host "安装包路径: $msiPath" -ForegroundColor Cyan
    Write-Host "安装包大小: $([math]::Round($msiSize, 2)) MB" -ForegroundColor Cyan
    Write-Host "" -ForegroundColor Green
    Write-Host "安装命令:" -ForegroundColor Yellow
    Write-Host "  msiexec /i \"$msiPath\" /quiet" -ForegroundColor Cyan
    Write-Host "卸载命令:" -ForegroundColor Yellow
    Write-Host "  msiexec /x \"$msiPath\" /quiet" -ForegroundColor Cyan
    
} catch {
    Write-Host "" -ForegroundColor Red
    Write-Host "=== 构建失败 ===" -ForegroundColor Red
    Write-Host "错误信息: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}