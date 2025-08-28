# 修复国家旗帜文件引用脚本
# 只保留存在的国家旗帜文件引用，注释掉其他所有引用

$flagsFile = "E:\workspace\gitee\rust\learn-rust\crates\domain_manager\src\countries\flags_pictures.rs"
$existingFlags = @("au", "ca", "cn", "de", "fr", "gb", "in", "jp", "kr", "us")

Write-Host "正在修复国家旗帜文件引用..."

# 读取文件内容
$content = Get-Content $flagsFile -Raw

# 为每个存在的国家创建正则表达式模式，确保它们不被注释
foreach ($flag in $existingFlags) {
    $upperFlag = $flag.ToUpper()
    # 确保存在的国家旗帜不被注释
    $pattern = "^\s*//\s*pub const $upperFlag"
    $replacement = "pub const $upperFlag"
    $content = $content -replace $pattern, $replacement
}

# 注释掉所有其他的国家旗帜引用（除了已存在的）
$allFlagsPattern = 'pub const ([A-Z]{2}): &\[u8\] = include_bytes!\("../../resources/countries_flags/4x3/([a-z]{2})\.svg"\);'
$content = [regex]::Replace($content, $allFlagsPattern, {
    param($match)
    $countryCode = $match.Groups[2].Value.ToLower()
    if ($existingFlags -contains $countryCode) {
        return $match.Value  # 保持不变
    } else {
        return "// " + $match.Value  # 添加注释
    }
})

# 写回文件
$content | Set-Content $flagsFile -NoNewline

Write-Host "国家旗帜文件引用修复完成！"
Write-Host "保留的国家: $($existingFlags -join ', ')"