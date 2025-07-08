pub mod provider;
pub mod domain;
pub mod dns_record;

pub use provider::Entity as Provider;
pub use domain::Entity as Domain;
pub use dns_record::Entity as DnsRecord;