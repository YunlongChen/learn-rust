#!/usr/bin/env pwsh
# Windows 打包脚本
# 用于构建 Windows 平台的 domain_manager 应用程序

param(
    [string]$BuildType = "release",
    [switch]$Clean = $false
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取脚本所在目录的父目录（项目根目录）
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "开始构建 Windows 版本..." -ForegroundColor Green
Write-Host "项目根目录: $ProjectRoot" -ForegroundColor Yellow
Write-Host "构建类型: $BuildType" -ForegroundColor Yellow

# 清理构建缓存（如果指定）
if ($Clean) {
    Write-Host "清理构建缓存..." -ForegroundColor Yellow
    cargo clean
}

# 检查 Rust 工具链
Write-Host "检查 Rust 工具链..." -ForegroundColor Yellow
rustc --version
cargo --version

# 添加 Windows 目标平台（如果尚未添加）
Write-Host "确保 Windows 目标平台已安装..." -ForegroundColor Yellow
rustup target add x86_64-pc-windows-msvc

# 构建应用程序
Write-Host "开始编译..." -ForegroundColor Yellow
if ($BuildType -eq "release") {
    $BuildResult = cargo build --release
    $BinaryPath = "..\..\target\release\domain_manager.exe"
} else {
    $BuildResult = cargo build
    $BinaryPath = "..\..\target\debug\domain_manager.exe"
}

# 检查编译是否成功（通过检查二进制文件是否存在）
if (!(Test-Path $BinaryPath)) {
    Write-Host "编译失败！未找到可执行文件: $BinaryPath" -ForegroundColor Red
    exit 1
}

# 检查构建是否成功
if (Test-Path $BinaryPath) {
    Write-Host "构建成功！" -ForegroundColor Green
    Write-Host "可执行文件位置: $BinaryPath" -ForegroundColor Green
    
    # 显示文件信息
    $FileInfo = Get-Item $BinaryPath
    Write-Host "文件大小: $([math]::Round($FileInfo.Length / 1MB, 2)) MB" -ForegroundColor Yellow
    
    # 创建发布目录
    $ReleaseDir = "release\windows-x64"
    if (!(Test-Path $ReleaseDir)) {
        New-Item -ItemType Directory -Path $ReleaseDir -Force | Out-Null
    }
    
    # 复制可执行文件
    Copy-Item $BinaryPath "$ReleaseDir\domain_manager.exe" -Force
    
    # 复制资源文件
    if (Test-Path "resources") {
        Write-Host "复制资源文件..." -ForegroundColor Yellow
        Copy-Item "resources" "$ReleaseDir\resources" -Recurse -Force
    }
    
    # 复制配置文件
    if (Test-Path "config") {
        Write-Host "复制配置文件..." -ForegroundColor Yellow
        Copy-Item "config" "$ReleaseDir\config" -Recurse -Force
    }
    
    # 复制本地化文件
    if (Test-Path "locales") {
        Write-Host "复制本地化文件..." -ForegroundColor Yellow
        Copy-Item "locales" "$ReleaseDir\locales" -Recurse -Force
    }
    
    # 创建启动脚本
    $StartScript = @'
@echo off
chcp 65001 > nul
echo 启动 Domain Manager...
start "" "domain_manager.exe"
'@
    $StartScript | Out-File -FilePath "$ReleaseDir\start.bat" -Encoding utf8
    
    Write-Host "打包完成！" -ForegroundColor Green
    Write-Host "发布目录: $ReleaseDir" -ForegroundColor Green
    
} else {
    Write-Host "构建失败！" -ForegroundColor Red
    exit 1
}

Write-Host "Windows 构建脚本执行完成。" -ForegroundColor Green