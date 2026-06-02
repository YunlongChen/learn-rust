# 修复country_utils.rs中的match语句
# 将所有不存在的国家旗帜常量替换为UNKNOWN

$countryUtilsFile = "E:\workspace\gitee\rust\learn-rust\crates\domain_manager\src\countries\country_utils.rs"
$existingFlags = @("AU", "CA", "CN", "DE", "FR", "GB", "IN", "JP", "KR", "US", "BOGON", "COMPUTER", "HOME", "UNKNOWN")

Write-Host "正在修复country_utils.rs中的match语句..."

# 读取文件内容
$content = Get-Content $countryUtilsFile -Raw

# 创建所有可能的国家代码列表（除了存在的）
$allCountryCodes = @(
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BM", "BN", "BO", "BR", "BS", "BT", "BV", "BW", "BY", "BZ",
    "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ",
    "DJ", "DK", "DM", "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET",
    "FI", "FJ", "FK", "FM", "FO", "GA", "GD", "GE", "GG", "GH", "GI", "GL", "GM", "GN", "GQ", "GR", "GS", "GT", "GU", "GW", "GY",
    "HK", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IO", "IQ", "IR", "IS", "IT",
    "JE", "JM", "JO", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KW", "KY", "KZ",
    "LA", "LB", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY",
    "MA", "MC", "MD", "ME", "MG", "MH", "MK", "ML", "MM", "MN", "MO", "MP", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ",
    "NA", "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ",
    "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PN", "PR", "PS", "PT", "PW", "PY",
    "QA", "RO", "RS", "RU", "RW", "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SK", "SL", "SM", "SN", "SO", "SR", "SS", "ST", "SV", "SX", "SY", "SZ",
    "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR", "TT", "TV", "TW", "TZ",
    "UA", "UG", "UY", "UZ", "VA", "VC", "VE", "VG", "VI", "VN", "VU", "WS", "YE", "ZA", "ZM", "ZW"
)

# 替换所有不存在的国家旗帜常量为UNKNOWN
foreach ($countryCode in $allCountryCodes) {
    if ($existingFlags -notcontains $countryCode) {
        # 替换单独的国家引用
        $pattern = "Country::$countryCode => $countryCode,"
        $replacement = "Country::$countryCode => UNKNOWN,"
        $content = $content -replace [regex]::Escape($pattern), $replacement
        
        # 替换在复合匹配中的引用（如 Country::XX => XX,）
        $pattern2 = "=> $countryCode,"
        $replacement2 = "=> UNKNOWN,"
        $content = $content -replace $pattern2, $replacement2
    }
}

# 写回文件
$content | Set-Content $countryUtilsFile -NoNewline

Write-Host "country_utils.rs修复完成！"
Write-Host "所有不存在的国家旗帜常量已替换为UNKNOWN"