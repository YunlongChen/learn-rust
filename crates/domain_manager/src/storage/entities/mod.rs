pub mod account;
pub mod dns_record;
pub mod domain;
pub mod provider;

pub use account::ActiveModel as AccountActiveModel;
pub use account::Entity as AccountEntity;
pub use dns_record::Entity as DnsRecordDbEntity;
pub use dns_record::Model as DnsRecordModal;
pub use domain::Entity as DomainDbEntity;
pub use domain::Model as DomainModal;
