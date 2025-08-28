#!/usr/bin/env pwsh
# 资源文件优化脚本
# 用于压缩和优化Domain Manager项目中的资源文件以减小打包体积

Param(
    [string]$ResourcesPath = "./resources",
    [switch]$DryRun = $false
)

Write-Host "=== Domain Manager 资源文件优化脚本 ===" -ForegroundColor Green
Write-Host "资源目录: $ResourcesPath" -ForegroundColor Yellow
Write-Host "干运行模式: $DryRun" -ForegroundColor Yellow
Write-Host ""

# 检查资源目录是否存在
if (-not (Test-Path $ResourcesPath)) {
    Write-Error "资源目录不存在: $ResourcesPath"
    exit 1
}

# 获取优化前的总大小
$beforeSize = (Get-ChildItem -Path $ResourcesPath -Recurse | Where-Object {!$_.PSIsContainer} | Measure-Object -Property Length -Sum).Sum
Write-Host "优化前总大小: $([math]::Round($beforeSize/1MB,2)) MB" -ForegroundColor Cyan

# 1. 优化SVG文件 - 移除不必要的元数据和空白
Write-Host "\n1. 优化SVG文件..." -ForegroundColor Yellow
$svgFiles = Get-ChildItem -Path $ResourcesPath -Filter "*.svg" -Recurse
Write-Host "找到 $($svgFiles.Count) 个SVG文件"

$svgSavedBytes = 0
foreach ($svgFile in $svgFiles) {
    $originalSize = $svgFile.Length
    $content = Get-Content $svgFile.FullName -Raw
    
    # 移除XML声明、注释和多余空白
    $optimizedContent = $content -replace '(?s)<!--.*?-->', '' `
                                -replace '\s+', ' ' `
                                -replace '> <', '><' `
                                -replace '^\s+|\s+$', ''
    
    if (-not $DryRun -and $optimizedContent.Length -lt $content.Length) {
        Set-Content -Path $svgFile.FullName -Value $optimizedContent -NoNewline
        $newSize = (Get-Item $svgFile.FullName).Length
        $saved = $originalSize - $newSize
        $svgSavedBytes += $saved
        Write-Host "  优化: $($svgFile.Name) - 节省 $saved 字节" -ForegroundColor Green
    } elseif ($DryRun) {
        $saved = $originalSize - $optimizedContent.Length
        if ($saved -gt 0) {
            $svgSavedBytes += $saved
            Write-Host "  [干运行] 可优化: $($svgFile.Name) - 可节省 $saved 字节" -ForegroundColor Gray
        }
    }
}

Write-Host "SVG优化完成，节省: $([math]::Round($svgSavedBytes/1KB,2)) KB" -ForegroundColor Green

# 2. 检查并建议移除不必要的资源
Write-Host "\n2. 分析可移除的资源..." -ForegroundColor Yellow

# 检查是否有未使用的国家旗帜
$flagsPath = Join-Path $ResourcesPath "countries_flags/4x3"
if (Test-Path $flagsPath) {
    $flagFiles = Get-ChildItem -Path $flagsPath -Filter "*.svg"
    Write-Host "发现 $($flagFiles.Count) 个国家旗帜文件"
    
    # 建议只保留常用的国家旗帜
    $commonCountries = @('cn', 'us', 'gb', 'de', 'fr', 'jp', 'kr', 'ca', 'au', 'in')
    $uncommonFlags = $flagFiles | Where-Object { $_.BaseName -notin $commonCountries }
    
    if ($uncommonFlags.Count -gt 0) {
        $uncommonSize = ($uncommonFlags | Measure-Object -Property Length -Sum).Sum
        Write-Host "  建议移除 $($uncommonFlags.Count) 个不常用国家旗帜，可节省: $([math]::Round($uncommonSize/1KB,2)) KB" -ForegroundColor Yellow
        
        if (-not $DryRun) {
            $response = Read-Host "是否移除不常用国家旗帜? (y/N)"
            if ($response -eq 'y' -or $response -eq 'Y') {
                foreach ($flag in $uncommonFlags) {
                    Remove-Item $flag.FullName -Force
                    Write-Host "    移除: $($flag.Name)" -ForegroundColor Red
                }
                Write-Host "  已移除不常用国家旗帜，节省: $([math]::Round($uncommonSize/1KB,2)) KB" -ForegroundColor Green
            }
        }
    }
}

# 3. 检查DB文件
Write-Host "\n3. 分析数据库文件..." -ForegroundColor Yellow
$dbPath = Join-Path $ResourcesPath "DB"
if (Test-Path $dbPath) {
    $dbFiles = Get-ChildItem -Path $dbPath
    foreach ($dbFile in $dbFiles) {
        Write-Host "  数据库文件: $($dbFile.Name) - $([math]::Round($dbFile.Length/1MB,2)) MB" -ForegroundColor Cyan
    }
    
    $totalDbSize = ($dbFiles | Measure-Object -Property Length -Sum).Sum
    Write-Host "  总DB文件大小: $([math]::Round($totalDbSize/1MB,2)) MB" -ForegroundColor Yellow
    Write-Host "  建议: 考虑使用在线API替代本地数据库文件" -ForegroundColor Yellow
}

# 4. 检查其他大文件
Write-Host "\n4. 检查其他大文件..." -ForegroundColor Yellow
$largeFiles = Get-ChildItem -Path $ResourcesPath -Recurse | Where-Object {!$_.PSIsContainer -and $_.Length -gt 100KB} | Sort-Object Length -Descending
foreach ($file in $largeFiles) {
    Write-Host "  大文件: $($file.Name) - $([math]::Round($file.Length/1KB,2)) KB" -ForegroundColor Cyan
}

# 获取优化后的总大小
$afterSize = (Get-ChildItem -Path $ResourcesPath -Recurse | Where-Object {!$_.PSIsContainer} | Measure-Object -Property Length -Sum).Sum
$totalSaved = $beforeSize - $afterSize

Write-Host "\n=== 优化结果 ===" -ForegroundColor Green
Write-Host "优化前大小: $([math]::Round($beforeSize/1MB,2)) MB" -ForegroundColor Cyan
Write-Host "优化后大小: $([math]::Round($afterSize/1MB,2)) MB" -ForegroundColor Cyan
Write-Host "节省空间: $([math]::Round($totalSaved/1KB,2)) KB ($([math]::Round(($totalSaved/$beforeSize)*100,2))%)" -ForegroundColor Green

if ($DryRun) {
    Write-Host "\n注意: 这是干运行模式，实际文件未被修改" -ForegroundColor Yellow
    Write-Host "要执行实际优化，请运行: .\optimize_resources.ps1" -ForegroundColor Yellow
}

Write-Host "\n优化完成!" -ForegroundColor Green