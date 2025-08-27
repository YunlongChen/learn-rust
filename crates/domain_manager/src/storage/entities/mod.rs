pub mod account;
pub mod dns_record;
pub mod domain;
pub mod provider;

pub use account::Model as AccountModel;
pub use domain::Model as DomainModel;
pub use domain::Entity as DomainDbEntity;
pub use dns_record::Model as DnsRecordModal;
pub use dns_record::Entity as DnsRecordDbEntity;