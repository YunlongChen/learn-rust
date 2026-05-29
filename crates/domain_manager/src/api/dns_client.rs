use crate::api::model::domain::DomainQueryResponse;
use crate::api::provider::aliyun::AliyunDnsClient;
use crate::gui::model::domain::DnsProvider::Aliyun;
use crate::gui::model::domain::{DnsProvider, Domain, DomainName, DomainStatus};
use crate::model::dns_record_response::Record;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use tracing::{error, info};

/// DNS客户端
#[async_trait]
pub trait DnsClientTrait {
    async fn list_domains(&self, page_num: u32, page_size: u32) -> Result<Vec<DomainName>>;

    async fn query_domain(&self, domain_name: &Domain) -> Result<DomainQueryResponse>;
    async fn list_dns_records(&self, domain_name: String) -> Result<Vec<Record>>;
    async fn add_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()>;
    async fn delete_dns_record(&self, domain_name: &DomainName, record_id: &str) -> Result<()>;
    async fn update_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()>;

    /// 验证凭证是否有效
    async fn validate_credentials(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DnsClient {
    _client: Client,
    pub access_key_id: String,
    pub access_key_secret: String,
    region_id: String,
    dns_provider: Vec<DnsProvider>,
}

impl DnsClient {
    /// 查询域名信息
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        region_id: String,
        dns_provider_vec: Vec<DnsProvider>,
    ) -> Self {
        Self {
            _client: Client::new(),
            access_key_id,
            access_key_secret,
            region_id,
            dns_provider: dns_provider_vec,
        }
    }

    pub async fn get_all_domain_info(&self) -> Result<Vec<Domain>> {
        let mut domain_name_list: Vec<Domain> = vec![];

        for dns_provider in &self.dns_provider {
            match dns_provider {
                Aliyun => {
                    let client = AliyunDnsClient::new(
                        self.access_key_id.clone(),
                        self.access_key_secret.clone(),
                    );
                    match client.list_domains(0, 100).await {
                        Ok(domain_names) => {
                            // 将结果添加到列表里面
                            domain_names.iter().for_each(|domain| {
                                domain_name_list.push(Domain {
                                    id: 0,
                                    name: domain.name.clone(),
                                    provider: Aliyun,
                                    status: DomainStatus::Active,
                                    expiry: "".to_string(),
                                    records: vec![],
                                })
                            })
                        }
                        Err(e) => {
                            error!("查询阿里云域名列表失败: {:?}", e);
                            return Err(e);
                        }
                    };
                }
                _ => {
                    error!("不支持的域名托管商：「{}」", dns_provider.name());
                }
            }
        }

        Ok(domain_name_list)
    }
}

#[async_trait]
impl DnsClientTrait for DnsClient {
    async fn list_domains(&self, _page_num: u32, _page_size: u32) -> Result<Vec<DomainName>> {
        let domains = self.get_all_domain_info().await?;
        let domain_names = domains
            .into_iter()
            .map(|domain| DomainName {
                name: domain.name,
                provider: domain.provider,
                ..DomainName::default()
            })
            .collect();
        Ok(domain_names)
    }

    async fn query_domain(&self, _domain_name: &Domain) -> Result<DomainQueryResponse> {
        // 这里需要根据具体的提供商实现域名查询
        // 目前暂不实现
        todo!("query_domain not implemented for DnsClient")
    }

    async fn list_dns_records(&self, domain_name: String) -> Result<Vec<Record>> {
        match self.dns_provider.first() {
            Some(Aliyun) => {
                let client = AliyunDnsClient::new(
                    self.access_key_id.clone(),
                    self.access_key_secret.clone(),
                );
                client.list_dns_records(domain_name).await
            }
            _ => Err(anyhow!("DNS records not implemented for this provider")),
        }
    }

    async fn add_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()> {
        match self.dns_provider.first() {
            Some(Aliyun) => {
                let client = AliyunDnsClient::new(
                    self.access_key_id.clone(),
                    self.access_key_secret.clone(),
                );
                client.add_dns_record(domain_name, record).await
            }
            _ => Err(anyhow!("DNS records not implemented for this provider")),
        }
    }

    async fn delete_dns_record(&self, domain_name: &DomainName, record_id: &str) -> Result<()> {
        match self.dns_provider.first() {
            Some(Aliyun) => {
                let client = AliyunDnsClient::new(
                    self.access_key_id.clone(),
                    self.access_key_secret.clone(),
                );
                client.delete_dns_record(domain_name, record_id).await
            }
            _ => Err(anyhow!("DNS records not implemented for this provider")),
        }
    }

    async fn update_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()> {
        match self.dns_provider.first() {
            Some(Aliyun) => {
                let client = AliyunDnsClient::new(
                    self.access_key_id.clone(),
                    self.access_key_secret.clone(),
                );
                client.update_dns_record(domain_name, record).await
            }
            _ => Err(anyhow!("DNS records not implemented for this provider")),
            None => Err(anyhow!("No DNS provider configured")),
        }
    }

    /// 验证凭证是否有效
    async fn validate_credentials(&self) -> Result<()> {
        // 通过查询域名列表来验证凭证
        // 如果凭证有效，应该能够成功获取域名列表
        // 如果凭证无效，API调用会失败
        match self.get_all_domain_info().await {
            Ok(domains) => {
                info!("凭证验证成功,查询到域名数量：{}", domains.len());
                Ok(())
            }
            Err(err) => {
                error!("凭证验证失败: {:?}", err);
                Err(err)
            }
        }
    }
}

impl Default for DnsClient {
    fn default() -> Self {
        Self {
            _client: Client::new(),
            access_key_id: "".to_string(),
            access_key_secret: "".to_string(),
            region_id: "".to_string(),
            dns_provider: vec![Aliyun],
        }
    }
}
