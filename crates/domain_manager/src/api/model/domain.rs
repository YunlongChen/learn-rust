use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct DomainQueryResponse {
    #[serde(rename = "PrePage")]
    pre_page: bool,

    #[serde(rename = "CurrentPageNum")]
    current_page_num: i32,

    #[serde(rename = "RequestId")]
    request_id: String,

    #[serde(rename = "PageSize")]
    page_size: i32,

    #[serde(rename = "TotalPageNum")]
    total_page_num: i32,

    #[serde(rename = "Data")]
    pub(crate) data: Data,

    #[serde(rename = "TotalItemNum")]
    total_item_num: i32,

    #[serde(rename = "NextPage")]
    next_page: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "Domain")]
    pub(crate) domain: Vec<Domain>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
    #[serde(rename = "RegistrantType")]
    registrant_type: String,

    #[serde(rename = "RegistrationDate")]
    registration_date: String,

    #[serde(rename = "ExpirationCurrDateDiff")]
    expiration_curr_date_diff: i64,

    #[serde(rename = "RegistrationDateLong")]
    registration_date_long: i64,

    #[serde(rename = "ResourceGroupId")]
    resource_group_id: String,

    #[serde(rename = "InstanceId")]
    instance_id: String,

    #[serde(rename = "DomainName")]
    pub(crate) domain_name: String,

    #[serde(rename = "Premium")]
    premium: bool,

    #[serde(rename = "ProductId")]
    product_id: String,

    #[serde(rename = "DomainAuditStatus")]
    domain_audit_status: String,

    #[serde(rename = "Ccompany")]
    ccompany: String,

    #[serde(rename = "ExpirationDateLong")]
    expiration_date_long: i64,

    #[serde(rename = "ExpirationDateStatus")]
    expiration_date_status: String,

    #[serde(rename = "DomainType")]
    domain_type: String,

    #[serde(rename = "ExpirationDate")]
    expiration_date: String,

    #[serde(rename = "ChgholderStatus")]
    chgholder_status: String,

    #[serde(rename = "Tag")]
    tag: DomainTag,

    #[serde(rename = "DomainStatus")]
    domain_status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DomainTag {
    #[serde(rename = "Tag")]
    tag: Vec<TagElement>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagElement {
    #[serde(rename = "Value")]
    value: String,

    #[serde(rename = "Key")]
    key: String,
}

impl TryFrom<String> for DomainQueryResponse {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}

impl TryFrom<Value> for DomainQueryResponse {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}
