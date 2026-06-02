#!/usr/bin/env pwsh
# 跨平台构建脚本
# 自动检测平台并调用相应的构建脚本

param(
    [string]$Platform = "auto",
    [string]$BuildType = "release",
    [switch]$Clean = $false,
    [switch]$Help = $false
)

# 显示帮助信息
if ($Help) {
    Write-Host "Domain Manager 跨平台构建脚本" -ForegroundColor Green
    Write-Host ""
    Write-Host "用法: .\scripts\build.ps1 [参数]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "参数:" -ForegroundColor Yellow
    Write-Host "  -Platform <平台>    指定构建平台 (auto, windows, linux, macos)" -ForegroundColor White
    Write-Host "  -BuildType <类型>   构建类型 (release, debug)" -ForegroundColor White
    Write-Host "  -Clean              清理构建缓存" -ForegroundColor White
    Write-Host "  -Help               显示此帮助信息" -ForegroundColor White
    Write-Host ""
    Write-Host "示例:" -ForegroundColor Yellow
    Write-Host "  .\scripts\build.ps1                    # 自动检测平台，构建release版本" -ForegroundColor White
    Write-Host "  .\scripts\build.ps1 -Platform windows  # 强制构建Windows版本" -ForegroundColor White
    Write-Host "  .\scripts\build.ps1 -BuildType debug   # 构建debug版本" -ForegroundColor White
    Write-Host "  .\scripts\build.ps1 -Clean             # 清理后构建" -ForegroundColor White
    exit 0
}

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取脚本所在目录
$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "=== Domain Manager 构建脚本 ===" -ForegroundColor Green
Write-Host "项目根目录: $ProjectRoot" -ForegroundColor Yellow
Write-Host "构建类型: $BuildType" -ForegroundColor Yellow

# 自动检测平台
if ($Platform -eq "auto") {
    if ($IsWindows -or $env:OS -eq "Windows_NT") {
        $Platform = "windows"
    } elseif ($IsLinux) {
        $Platform = "linux"
    } elseif ($IsMacOS) {
        $Platform = "macos"
    } else {
        # 尝试通过其他方式检测
        $OSInfo = [System.Environment]::OSVersion.Platform
        if ($OSInfo -eq "Win32NT") {
            $Platform = "windows"
        } else {
            Write-Host "无法自动检测平台，请手动指定 -Platform 参数" -ForegroundColor Red
            exit 1
        }
    }
}

Write-Host "检测到平台: $Platform" -ForegroundColor Yellow

# 切换到项目根目录
Set-Location $ProjectRoot

# 检查 Rust 环境
Write-Host "检查 Rust 环境..." -ForegroundColor Yellow
try {
    $RustVersion = rustc --version
    $CargoVersion = cargo --version
    Write-Host "Rust: $RustVersion" -ForegroundColor Green
    Write-Host "Cargo: $CargoVersion" -ForegroundColor Green
} catch {
    Write-Host "错误: 未找到 Rust 工具链，请先安装 Rust" -ForegroundColor Red
    Write-Host "访问 https://rustup.rs/ 安装 Rust" -ForegroundColor Yellow
    exit 1
}

# 根据平台调用相应的构建脚本
switch ($Platform) {
    "windows" {
        Write-Host "检测到 Windows 系统，调用 Windows 构建脚本..." -ForegroundColor Green
        $scriptPath = Join-Path $PSScriptRoot "build-windows.ps1"
        & $scriptPath @PSBoundParameters
    }
    
    "linux" {
        Write-Host "检测到 Linux/macOS 系统，调用 Linux 构建脚本..." -ForegroundColor Green
        $scriptPath = Join-Path $PSScriptRoot "build-linux.sh"
        & bash $scriptPath @args
    }
    
    "macos" {
        Write-Host "macOS 构建暂未实现" -ForegroundColor Yellow
        Write-Host "请使用以下命令手动构建:" -ForegroundColor Yellow
        Write-Host "  rustup target add x86_64-apple-darwin" -ForegroundColor White
        Write-Host "  cargo build --release --target x86_64-apple-darwin" -ForegroundColor White
        exit 1
    }
    
    default {
        Write-Host "错误: 不支持的平台: $Platform" -ForegroundColor Red
        Write-Host "支持的平台: windows, linux, macos" -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "" 
Write-Host "=== 构建完成 ===" -ForegroundColor Green
Write-Host "查看构建结果: .\release\" -ForegroundColor Yellow