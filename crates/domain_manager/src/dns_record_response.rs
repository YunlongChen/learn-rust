// YApi QuickType插件生成，具体参考文档:https://plugins.jetbrains.com/plugin/18847-yapi-quicktype/documentation

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsRecordResponse {
    #[serde(rename = "TotalCount")]
    total_count: i64,

    #[serde(rename = "PageSize")]
    page_size: i64,

    #[serde(rename = "RequestId")]
    request_id: String,

    #[serde(rename = "DomainRecords")]
    pub(crate) domain_records: DomainRecords,

    #[serde(rename = "PageNumber")]
    page_number: i64,
}

impl From<String> for DnsRecordResponse {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DomainRecords {
    #[serde(rename = "Record")]
    pub record: Vec<Record>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    #[serde(rename = "Status")]
    status: Status,

    #[serde(rename = "RR")]
    pub(crate) rr: String,

    #[serde(rename = "Line")]
    line: Line,

    #[serde(rename = "Locked")]
    locked: bool,

    #[serde(rename = "Type")]
    pub(crate) record_type: Type,

    #[serde(rename = "Value")]
    pub(crate) value: String,

    #[serde(rename = "RecordId")]
    pub(crate) record_id: String,

    #[serde(rename = "UpdateTimestamp")]
    update_timestamp: Option<i64>,

    #[serde(rename = "TTL")]
    ttl: i64,

    #[serde(rename = "CreateTimestamp")]
    create_timestamp: i64,

    #[serde(rename = "Weight")]
    weight: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Line {
    #[serde(rename = "default")]
    Default,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    #[serde(rename = "A")]
    A,

    #[serde(rename = "CNAME")]
    Cname,

    MX,
    AAAA,
    TXT,
    NS,
    SOA,
    PTR,
    SRV,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::A => write!(f, "A"),
            Type::Cname => write!(f, "CNAME"),
            Type::MX => write!(f, "MX"),
            Type::AAAA => write!(f, "AAAA"),
            Type::TXT => write!(f, "TXT"),
            Type::NS => write!(f, "NS"),
            Type::SOA => write!(f, "SOA"),
            Type::PTR => write!(f, "PTR"),
            Type::SRV => write!(f, "SRV"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Status {
    #[serde(rename = "ENABLE")]
    Enable,
}

#[cfg(test)]
mod tests {
    use crate::dns_record_response::{DnsRecordResponse, Status};

    #[tokio::test]
    async fn test_get_text() {
        // The type of `j` is `&str`
        let json_str = r#" 
        {
          "TotalCount": 24,
          "PageSize": 20,
          "RequestId": "045C068A-0268-5FF8-B7A1-5C66B873D31F",
          "DomainRecords": {
            "Record": [
              {
                "Status": "ENABLE",
                "RR": "fnos",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "192.168.9.103",
                "RecordId": "935051498758138880",
                "UpdateTimestamp": 1734702274098,
                "TTL": 600,
                "CreateTimestamp": 1734702273965,
                "Weight": 1
              },
            {
                    "Status": "ENABLE",
                    "RR": "@",
                    "Line": "default",
                    "Locked": false,
                    "Type": "MX",
                    "DomainName": "stanic.xyz",
                    "Priority": 10,
                    "Value": "mxbiz2.qq.com",
                    "RecordId": "846521046484463616",
                    "UpdateTimestamp": 1692487665000,
                    "TTL": 600,
                    "CreateTimestamp": 1692487665000
                  },
              {
                "Status": "ENABLE",
                "RR": "dashy",
                "Line": "default",
                "Locked": false,
                "Type": "CNAME",
                "DomainName": "chenyunlong.cn",
                "Value": "dashy.b0.aicdn.com",
                "RecordId": "934141464568209408",
                "UpdateTimestamp": 1734268335981,
                "TTL": 600,
                "CreateTimestamp": 1734268335801,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "jenkins",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "180.88.231.219",
                "RecordId": "871207945518320640",
                "UpdateTimestamp": 1710191068000,
                "TTL": 600,
                "CreateTimestamp": 1704259296000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "kubernetes",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "852247766315446272",
                "UpdateTimestamp": 1695218378000,
                "TTL": 600,
                "CreateTimestamp": 1695218378000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "acme",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "851694784907677696",
                "UpdateTimestamp": 1694954696000,
                "TTL": 600,
                "CreateTimestamp": 1694954696000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "consul",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "849709564962583552",
                "UpdateTimestamp": 1694008069000,
                "TTL": 600,
                "CreateTimestamp": 1694008069000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "start",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "845731565756007424",
                "UpdateTimestamp": 1692111211000,
                "TTL": 600,
                "CreateTimestamp": 1692111211000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "@",
                "Line": "default",
                "Locked": false,
                "Type": "CNAME",
                "DomainName": "chenyunlong.cn",
                "Value": "halo-chenyunlong.b0.aicdn.com",
                "RecordId": "816144082422210560",
                "UpdateTimestamp": 1699456751000,
                "TTL": 600,
                "CreateTimestamp": 1678002800000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "resume",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "121.36.163.95",
                "RecordId": "795856988257362944",
                "TTL": 600,
                "CreateTimestamp": 1668329159000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "dev",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "121.36.163.95",
                "RecordId": "795816027756112896",
                "TTL": 600,
                "CreateTimestamp": 1668309628000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "img",
                "Line": "default",
                "Locked": false,
                "Type": "CNAME",
                "DomainName": "chenyunlong.cn",
                "Value": "stanic-image-cloud.b0.aicdn.com",
                "RecordId": "765110698708022272",
                "TTL": 600,
                "CreateTimestamp": 1653668186000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "love-story",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "121.36.163.95",
                "RecordId": "754043414887069696",
                "TTL": 600,
                "CreateTimestamp": 1648390894000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "weixin",
                "Line": "default",
                "Locked": false,
                "Type": "CNAME",
                "DomainName": "chenyunlong.cn",
                "Value": "sh0002.gw.tencentcloudbase.com",
                "RecordId": "734093660777169920",
                "TTL": 600,
                "CreateTimestamp": 1638878110000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "jenkins",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "223.208.63.213",
                "RecordId": "733163637022922752",
                "UpdateTimestamp": 1704113555000,
                "TTL": 600,
                "CreateTimestamp": 1638434640000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "nameit",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "21412514121199616",
                "UpdateTimestamp": 1692439260000,
                "TTL": 600,
                "CreateTimestamp": 1615564011000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "admin",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "20879056739777536",
                "UpdateTimestamp": 1692439254000,
                "TTL": 600,
                "CreateTimestamp": 1607424097000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "api",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "20827872566970368",
                "UpdateTimestamp": 1692439013000,
                "TTL": 600,
                "CreateTimestamp": 1606643089000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "sakurasou",
                "Line": "default",
                "Locked": false,
                "Type": "A",
                "DomainName": "chenyunlong.cn",
                "Value": "1.15.242.253",
                "RecordId": "20816149640524800",
                "UpdateTimestamp": 1692439283000,
                "TTL": 600,
                "CreateTimestamp": 1606464211000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "bangumi",
                "Line": "default",
                "Locked": false,
                "Type": "CNAME",
                "DomainName": "chenyunlong.cn",
                "Value": "bangumi-cloud.b0.aicdn.com",
                "RecordId": "20687800534051840",
                "UpdateTimestamp": 1699468169000,
                "TTL": 600,
                "CreateTimestamp": 1604505759000,
                "Weight": 1
              },
              {
                "Status": "ENABLE",
                "RR": "status",
                "Line": "default",
                "Locked": false,
                "Type": "CNAME",
                "DomainName": "chenyunlong.cn",
                "Value": "stats.uptimerobot.com",
                "RecordId": "20635271589544960",
                "TTL": 600,
                "CreateTimestamp": 1603704231000,
                "Weight": 1
              }
            ]
          },
          "PageNumber": 1
        }"#;

        let u: DnsRecordResponse = serde_json::from_str(json_str).unwrap();
        println!("{:#?}", u);
        assert_eq!(u.domain_records.record[0].rr, "fnos");
        assert_eq!(u.domain_records.record[0].value, "192.168.9.103");
        assert_eq!(u.domain_records.record[0].status, Status::Enable);
    }
}
