use crate::api::dns_client::DnsClientTrait;
use crate::api::model::domain::DomainQueryResponse;
use crate::gui::model::domain::{DnsProvider, Domain, DomainName};
use crate::model::dns_record_response::{Record, Status, Type};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Cloudflare API响应结构体
#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    messages: Vec<String>,
    result: T,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    code: u32,
    message: String,
}

/// Cloudflare Zone结构体
#[derive(Debug, Deserialize)]
struct CloudflareZone {
    id: String,
    name: String,
    status: String,
}

/// Cloudflare DNS记录结构体
#[derive(Debug, Deserialize, Serialize)]
struct CloudflareDnsRecord {
    id: Option<String>,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
    ttl: u32,
    priority: Option<u32>,
    proxied: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct CloudflareDnsClient {
    client: Client,
    api_token: String,
    base_url: String,
}

impl CloudflareDnsClient {
    /// 创建新的Cloudflare DNS客户端
    ///
    /// # 参数
    /// * `api_token` - Cloudflare API Token
    /// * `_email` - 用户邮箱（当前版本使用API Token，此参数保留兼容性）
    pub fn new(api_token: String, _email: String) -> Result<Self> {
        let client = Client::new();
        let base_url = "https://api.cloudflare.com/client/v4".to_string();

        Ok(Self {
            client,
            api_token,
            base_url,
        })
    }

    /// 获取Zone ID通过域名
    async fn get_zone_id(&self, domain_name: &str) -> Result<String> {
        info!("正在获取域名 {} 的Zone ID...", domain_name);

        let url = format!("{}/zones?name={}", self.base_url, domain_name);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取Zone ID失败: HTTP {}", response.status()));
        }

        let cf_response: CloudflareResponse<Vec<CloudflareZone>> = response.json().await?;

        if !cf_response.success {
            return Err(anyhow!("Cloudflare API错误: {:?}", cf_response.errors));
        }

        if let Some(zone) = cf_response.result.first() {
            Ok(zone.id.clone())
        } else {
            Err(anyhow!("未找到域名对应的Zone: {}", domain_name))
        }
    }

    /// 将Cloudflare DNS记录转换为内部Record格式
    fn convert_cf_record_to_internal(cf_record: &CloudflareDnsRecord) -> Record {
        let record_type = match cf_record.record_type.as_str() {
            "A" => Type::A,
            "AAAA" => Type::AAAA,
            "CNAME" => Type::Cname,
            "MX" => Type::MX,
            "TXT" => Type::TXT,
            _ => Type::A, // 默认类型
        };

        Record::new(
            Status::Enable,
            cf_record.name.clone(),
            record_type,
            cf_record.content.clone(),
            cf_record.id.clone().unwrap_or_default(),
            cf_record.ttl as i32,
        )
    }

    /// 将内部Record格式转换为Cloudflare DNS记录
    fn convert_internal_to_cf_record(record: &Record) -> CloudflareDnsRecord {
        let record_type = match record.record_type {
            Type::A => "A",
            Type::AAAA => "AAAA",
            Type::Cname => "CNAME",
            Type::MX => "MX",
            Type::TXT => "TXT",
            _ => "A", // 默认类型
        };

        let mut priority = None;
        let mut content = record.value.clone();

        // 对于MX记录，需要特殊处理priority
        if record.record_type == Type::MX {
            if let Some(space_pos) = record.value.find(' ') {
                if let Ok(prio) = record.value[..space_pos].parse::<u32>() {
                    priority = Some(prio);
                    content = record.value[space_pos + 1..].to_string();
                }
            }
        }

        CloudflareDnsRecord {
            id: if record.record_id.is_empty() {
                None
            } else {
                Some(record.record_id.clone())
            },
            name: record.rr.clone(),
            record_type: record_type.to_string(),
            content,
            ttl: record.ttl as u32,
            priority,
            proxied: Some(false),
        }
    }
}

#[async_trait]
impl DnsClientTrait for CloudflareDnsClient {
    /// 获取域名列表（Cloudflare中为Zone列表）
    async fn list_domains(&self, _page_num: u32, _page_size: u32) -> Result<Vec<DomainName>> {
        info!("正在获取Cloudflare域名列表...");

        let url = format!("{}/zones", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取域名列表失败: HTTP {}", response.status()));
        }

        let cf_response: CloudflareResponse<Vec<CloudflareZone>> = response.json().await?;

        if !cf_response.success {
            return Err(anyhow!("Cloudflare API错误: {:?}", cf_response.errors));
        }

        let mut domain_list = Vec::new();
        for zone in cf_response.result {
            domain_list.push(DomainName {
                name: zone.name,
                provider: DnsProvider::CloudFlare,
                ..DomainName::default()
            });
        }

        info!("获取到 {} 个Cloudflare域名", domain_list.len());
        Ok(domain_list)
    }

    /// 查询域名信息
    async fn query_domain(&self, _domain_name: &Domain) -> Result<DomainQueryResponse> {
        // Cloudflare的域名查询功能暂未实现
        // 可以根据需要实现Zone详细信息查询
        todo!("Cloudflare域名查询功能待实现")
    }

    /// 获取DNS记录列表
    async fn list_dns_records(&self, domain_name: String) -> Result<Vec<Record>> {
        info!("正在获取域名 {} 的DNS记录...", domain_name);

        // 首先获取Zone ID
        let zone_id = self.get_zone_id(&domain_name).await?;

        // 获取DNS记录
        let url = format!("{}/zones/{}/dns_records", self.base_url, zone_id);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取DNS记录失败: HTTP {}", response.status()));
        }

        let cf_response: CloudflareResponse<Vec<CloudflareDnsRecord>> = response.json().await?;

        if !cf_response.success {
            return Err(anyhow!("Cloudflare API错误: {:?}", cf_response.errors));
        }

        let mut records = Vec::new();
        for cf_record in cf_response.result {
            records.push(Self::convert_cf_record_to_internal(&cf_record));
        }

        info!("获取到 {} 条DNS记录", records.len());
        Ok(records)
    }

    /// 添加DNS记录
    async fn add_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()> {
        info!("正在添加DNS记录: {} -> {}", record.rr, record.value);

        // 获取Zone ID
        let zone_id = self.get_zone_id(&domain_name.name).await?;

        // 转换记录
        let cf_record = Self::convert_internal_to_cf_record(record);

        // 创建DNS记录
        let url = format!("{}/zones/{}/dns_records", self.base_url, zone_id);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&cf_record)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("添加DNS记录失败: HTTP {}", response.status()));
        }

        let cf_response: CloudflareResponse<CloudflareDnsRecord> = response.json().await?;

        if !cf_response.success {
            return Err(anyhow!("Cloudflare API错误: {:?}", cf_response.errors));
        }

        info!("DNS记录添加成功: {:?}", cf_response.result.id);
        Ok(())
    }

    /// 删除DNS记录
    async fn delete_dns_record(&self, domain_name: &DomainName, record_id: &str) -> Result<()> {
        info!("正在删除DNS记录: {}", record_id);

        // 获取Zone ID
        let zone_id = self.get_zone_id(&domain_name.name).await?;

        // 删除DNS记录
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            self.base_url, zone_id, record_id
        );
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("删除DNS记录失败: HTTP {}", response.status()));
        }

        info!("DNS记录删除成功");
        Ok(())
    }

    /// 更新DNS记录
    async fn update_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()> {
        info!("正在更新DNS记录: {} -> {}", record.rr, record.value);

        // 获取Zone ID
        let zone_id = self.get_zone_id(&domain_name.name).await?;

        // 转换记录
        let cf_record = Self::convert_internal_to_cf_record(record);

        // 更新DNS记录
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            self.base_url, zone_id, record.record_id
        );
        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&cf_record)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("更新DNS记录失败: HTTP {}", response.status()));
        }

        let cf_response: CloudflareResponse<CloudflareDnsRecord> = response.json().await?;

        if !cf_response.success {
            return Err(anyhow!("Cloudflare API错误: {:?}", cf_response.errors));
        }

        info!("DNS记录更新成功: {:?}", cf_response.result.id);
        Ok(())
    }

    /// 验证凭证是否有效
    async fn validate_credentials(&self) -> Result<()> {
        // 通过查询Zone列表来验证凭证
        // 如果凭证有效，应该能够成功获取Zone列表
        // 如果凭证无效，API调用会失败
        match self.list_domains(1, 1).await {
            Ok(_) => {
                info!("Cloudflare凭证验证成功");
                Ok(())
            }
            Err(err) => {
                error!("Cloudflare凭证验证失败: {:?}", err);
                Err(err)
            }
        }
    }
}
