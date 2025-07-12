use crate::api::dns_client::DnsClientTrait;
use crate::api::model::domain::DomainQueryResponse;
use crate::gui::model::domain::DnsProvider::Aliyun;
use crate::gui::model::domain::{Domain, DomainName};
use crate::model::dns_record_response::{DnsRecordResponse, Record};
use domain_client::RequestBody;
use reqwest::{Client, Method};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use tracing::info;

#[derive(Debug, Clone)]
pub struct AliyunDnsClient {
    client: Client,
    access_key_id: String,
    access_key_secret: String,
    region_id: String,
}

impl AliyunDnsClient {
    pub fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            client: Client::new(),
            access_key_id,
            access_key_secret,
            region_id: "cn-qingdao".to_string(), // 默认区域
        }
    }

    pub fn with_region(mut self, region_id: String) -> Self {
        self.region_id = region_id;
        self
    }

    /// 内部调用API的通用方法
    ///
    async fn call_ali_api(
        &self,
        method: Method,
        host: &str,
        canonical_uri: &str,
        query_params: &[(&str, &str)],
        action: &str,
        version: &str,
        body: RequestBody,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        info!(
            "请求后台接口，当前密钥：「{:?}」，请求参数：「{:?}」",
            &self.access_key_id,
            &body
        );
        let response = domain_client::call_api(
            self.client.clone(),
            method,
            host,
            canonical_uri,
            query_params,
            action,
            version,
            body,
            &self.access_key_id,
            &self.access_key_secret,
        )
        .await?;
        info!("接口请求结果:{}", json!(&response));
        Ok(serde_json::from_str(&response)?)
    }
}

impl DnsClientTrait for AliyunDnsClient {
    /// 查询域名列表
    async fn list_domains(
        &self,
        page_num: u32,
        page_size: u32,
    ) -> Result<Vec<DomainName>, Box<dyn Error>> {
        let query_params = &[
            ("RegionId", self.region_id.as_str()),
            ("PageNum", &page_num.to_string()),
            ("PageSize", &page_size.to_string()),
        ];

        let mut body = HashMap::new();
        body.insert("PageNum".to_string(), json!(page_num));
        body.insert("PageSize".to_string(), json!(page_size));

        let response = self
            .call_ali_api(
                Method::GET,
                "domain.aliyuncs.com",
                "/",
                query_params,
                "QueryDomainList",
                "2018-01-29",
                RequestBody::Json(body),
            )
            .await?;

        info!("调用结果：{:?}", response);
        let result: Result<DomainQueryResponse, serde_json::Error> = response.try_into();
        match result {
            Ok(response) => {
                let domain_list = response.data.domain;
                info!("查询结果：{:?}", domain_list);

                let mut output = Vec::new();
                // Pre-reserve the memory, exiting if we can't
                output.try_reserve(domain_list.len())?;
                // Now we know this can't OOM in the middle of our complex work
                output.extend(domain_list.iter().map(|domain| DomainName {
                    name: domain.domain_name.clone(),
                    provider: Aliyun,
                    ..DomainName::default()
                }));
                // let domain_list = query_aliyun_domain_list();
                Ok(output)
            }
            Err(err) => {
                info!("解析结果异常：{:?}", err);
                Ok(vec![])
            }
        }
    }

    fn query_domain(&self, _domain: &Domain) -> Result<DomainQueryResponse, Box<dyn Error>> {
        todo!()
    }

    /// 查询DNS记录
    async fn list_dns_records(&self, domain_name: String) -> Result<Vec<Record>, Box<dyn Error>> {
        let query_params = &[
            ("RegionId", self.region_id.as_str()),
            ("DomainName", domain_name.as_str()),
            ("PageSize", "100"),
        ];

        let mut body = HashMap::new();
        body.insert("PageSize".to_string(), json!(100));

        let response = self
            .call_ali_api(
                Method::GET,
                "dns.aliyuncs.com",
                "/",
                query_params,
                "DescribeDomainRecords",
                "2015-01-09",
                RequestBody::Json(body),
            )
            .await?;

        info!("调用结果：{:?}", response);

        let result: Result<DnsRecordResponse, serde_json::Error> = response.try_into();
        match result {
            Ok(result) => Ok(result.domain_records.record),
            Err(err) => {
                info!("调用失败：{:?}", err);
                Err(err.into())
            }
        }
    }

    fn add_dns_record(
        &self,
        domain_name: &DomainName,
        record: &Record,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn delete_dns_record(
        &self,
        domain_name: &DomainName,
        record_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn update_dns_record(
        &self,
        domain_name: &DomainName,
        record: &Record,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
