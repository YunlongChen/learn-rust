use crate::api::dns_client::DnsClientTrait;
use crate::api::model::domain::DomainQueryResponse;
use crate::gui::model::domain::{Domain, DomainName};
use crate::model::dns_record_response::Record;
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::Environment;
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct CloudflareDnsClient {
    client: Client,
    access_key_id: String,
    access_key_secret: String,
    region_id: String,
}

impl CloudflareDnsClient {
    pub fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            client: Client::new(),
            access_key_id,
            access_key_secret,
            region_id: "".to_string(),
        }
    }
}

impl DnsClientTrait for CloudflareDnsClient {
    async fn list_domains(
        self: &Self,
        page_num: u32,
        page_size: u32,
    ) -> Result<Vec<DomainName>, Box<dyn Error>> {
        let environment = Environment::Custom("https://api.cloudflare.com/client/v4".to_string());
        let credentials = Credentials::UserAuthToken {
            token: "YOUR_API_TOKEN".into(),
        };
        let config = ClientConfig::default();
        let client =
            cloudflare::framework::client::async_api::Client::new(credentials, config, environment);

        match client {
            Ok(domain_client) => Ok(vec![]),
            Err(err) => Err(Box::new(err)),
        }
    }

    fn query_domain(
        &self,
        domain_name: &Domain,
    ) -> Result<DomainQueryResponse, Box<dyn Error>> {
        todo!()
    }

    async fn list_dns_records(
        self: &Self,
        domain_name: String,
    ) -> Result<Vec<Record>, Box<dyn Error>> {
        todo!()
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
