use crate::api::model::domain::DomainQueryResponse;
use crate::api::provider::aliyun::AliyunDnsClient;
use crate::gui::model::domain::DnsProvider::Aliyun;
use crate::gui::model::domain::{DnsProvider, Domain, DomainName};
use crate::model::dns_record_response::Record;
use reqwest::Client;
use std::error::Error;
use log::{error};

/// DNS客户端
pub trait DnsClientTrait {
    async fn list_domains(
        self: &Self,
        page_num: u32,
        page_size: u32,
    ) -> Result<Vec<DomainName>, Box<dyn Error>>;

    fn query_domain(&self, domain_name: &Domain) -> Result<DomainQueryResponse, Box<dyn Error>>;
    async fn list_dns_records(
        self: &Self,
        domain_name: String,
    ) -> Result<Vec<Record>, Box<dyn Error>>;
    fn add_dns_record(
        &self,
        domain_name: &DomainName,
        record: &Record,
    ) -> Result<(), Box<dyn Error>>;
    fn delete_dns_record(
        &self,
        domain_name: &DomainName,
        record_id: &str,
    ) -> Result<(), Box<dyn Error>>;
    fn update_dns_record(
        &self,
        domain_name: &DomainName,
        record: &Record,
    ) -> Result<(), Box<dyn Error>>;
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

    pub async fn get_all_domain_info(&self) -> Result<Vec<Domain>, Box<dyn Error>> {
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
                                    name: domain.name.clone(),
                                    ..Default::default()
                                })
                            })
                        }
                        Err(_) => {}
                    };
                }
                _ => {
                    error!("不支持的域名托管商：「{}」",dns_provider.name());
                }
            }
        }

        Ok(domain_name_list)
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
