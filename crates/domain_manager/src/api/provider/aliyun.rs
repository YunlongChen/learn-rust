use crate::api::dns_client::DnsClientTrait;
use crate::api::model::domain::DomainQueryResponse;
use crate::gui::model::domain::DnsProvider::Aliyun;
use crate::gui::model::domain::{Domain, DomainName};
use crate::model::dns_record_response::{DnsRecordResponse, Record};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use domain_client::RequestBody;
use reqwest::{Client, Method};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

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
    ) -> Result<serde_json::Value> {
        info!(
            "请求后台接口，当前密钥：「{:?}」，请求参数：「{:?}」",
            &self.access_key_id, &body
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
        .await;
        info!("接口请求结果:{}", json!(&response));

        let resp_str = response.unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&resp_str)?;

        if let Some(code) = json_val.get("Code") {
            let message = json_val
                .get("Message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            let request_id = json_val
                .get("RequestId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return Err(anyhow!(
                "Aliyun API Error: {} - {} (RequestId: {})",
                code,
                message,
                request_id
            ));
        }

        Ok(json_val)
    }
}

#[async_trait]
impl DnsClientTrait for AliyunDnsClient {
    /// 查询域名列表
    async fn list_domains(&self, page_num: u32, page_size: u32) -> Result<Vec<DomainName>> {
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

        info!("调用结果：{:?}", serde_json::to_string(&response));
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
                error!("解析结果异常：{:?}", err);
                Err(anyhow!("解析结果异常: HTTP {}", err.to_string()))
            }
        }
    }

    async fn query_domain(&self, _domain: &Domain) -> Result<DomainQueryResponse> {
        todo!()
    }

    /// 查询DNS记录
    async fn list_dns_records(&self, domain_name: String) -> Result<Vec<Record>> {
        let query_params = &[
            ("RegionId", self.region_id.as_str()),
            ("DomainName", domain_name.as_str()),
            ("PageSize", "100"),
        ];

        let mut body = HashMap::new();
        body.insert("PageSize".to_string(), json!(100));

        let response_result = self
            .call_ali_api(
                Method::GET,
                "dns.aliyuncs.com",
                "/",
                query_params,
                "DescribeDomainRecords",
                "2015-01-09",
                RequestBody::Json(body),
            )
            .await
            .map_err(|err| {
                error!("获取域名解析列表发生了异常：「{:?}」", err);
                err.to_string()
            });

        info!("调用结果：{:?}", response_result);

        let result: Result<DnsRecordResponse, serde_json::Error> =
            response_result.unwrap().try_into();
        match result {
            Ok(result) => Ok(result.domain_records.record),
            Err(err) => {
                info!("反序列化结果异常：{:?}", err);
                Err(err.into())
            }
        }
    }

    /// 添加DNS记录
    async fn add_dns_record(&self, domain_name: &DomainName, record: &Record) -> Result<()> {
        let query_params = &[
            ("RegionId", self.region_id.as_str()),
            ("DomainName", domain_name.name.as_str()),
            ("RR", record.rr.as_str()),
            ("Type", record.record_type.get_value()),
            ("Value", record.value.as_str()),
            ("TTL", &record.ttl.to_string()),
        ];

        let mut body = HashMap::new();
        body.insert("DomainName".to_string(), json!(domain_name.name));
        body.insert("RR".to_string(), json!(record.rr));
        body.insert("Type".to_string(), json!(record.record_type.get_value()));
        body.insert("Value".to_string(), json!(record.value));
        body.insert("TTL".to_string(), json!(record.ttl));

        let response = self
            .call_ali_api(
                Method::POST,
                "dns.aliyuncs.com",
                "/",
                query_params,
                "AddDomainRecord",
                "2015-01-09",
                RequestBody::Json(body),
            )
            .await?;

        info!("添加DNS记录结果：{:?}", response);
        Ok(())
    }

    /// 删除DNS记录
    async fn delete_dns_record(&self, _domain_name: &DomainName, record_id: &str) -> Result<()> {
        let query_params = &[
            ("RegionId", self.region_id.as_str()),
            ("RecordId", record_id),
        ];

        let mut body = HashMap::new();
        body.insert("RecordId".to_string(), json!(record_id));

        let response = self
            .call_ali_api(
                Method::POST,
                "dns.aliyuncs.com",
                "/",
                query_params,
                "DeleteDomainRecord",
                "2015-01-09",
                RequestBody::Json(body),
            )
            .await?;

        info!("删除DNS记录结果：{:?}", response);
        Ok(())
    }

    /// 更新DNS记录
    async fn update_dns_record(&self, _domain_name: &DomainName, record: &Record) -> Result<()> {
        let query_params = &[
            ("RegionId", self.region_id.as_str()),
            ("RecordId", record.record_id.as_str()),
            ("RR", record.rr.as_str()),
            ("Type", record.record_type.get_value()),
            ("Value", record.value.as_str()),
            ("TTL", &record.ttl.to_string()),
        ];

        let mut body = HashMap::new();
        body.insert("RecordId".to_string(), json!(record.record_id));
        body.insert("RR".to_string(), json!(record.rr));
        body.insert("Type".to_string(), json!(record.record_type.get_value()));
        body.insert("Value".to_string(), json!(record.value));
        body.insert("TTL".to_string(), json!(record.ttl));

        let response = self
            .call_ali_api(
                Method::POST,
                "dns.aliyuncs.com",
                "/",
                query_params,
                "UpdateDomainRecord",
                "2015-01-09",
                RequestBody::Json(body),
            )
            .await?;

        info!("更新DNS记录结果：{:?}", response);
        Ok(())
    }

    /// 验证凭证是否有效
    async fn validate_credentials(&self) -> Result<()> {
        // 通过查询域名列表来验证凭证
        // 如果凭证有效，应该能够成功获取域名列表
        // 如果凭证无效，API调用会失败
        match self.list_domains(1, 1).await {
            Ok(_) => {
                info!("凭证验证成功");
                Ok(())
            }
            Err(err) => {
                error!("凭证验证失败: {:?}", err);
                Err(err)
            }
        }
    }
}
