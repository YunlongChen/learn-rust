use crate::models::domain::DomainEntity;
use crate::models::record::{NewRecord, RecordEntity};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use std::error::Error;

/// 添加新域名
pub fn add_record(
    conn: &DatabaseConnection,
    new_record: NewRecord,
    domain_entity: &DomainEntity,
) -> Result<RecordEntity, Box<dyn Error>> {
    let now = Utc::now().to_string();

    // let record_id = conn.last_insert_rowid();

    Ok(RecordEntity {
        // id: record_id,
        id: 0,
        account_id: new_record.account_id,
        domain_id: domain_entity.id,
        record_name: new_record.record_name,
        record_type: new_record.record_type,
        record_value: new_record.record_value,
        ttl: new_record.ttl,
    })
}

/// 更新域名信息
pub fn update_domain(
    conn: &DatabaseConnection,
    domain: &DomainEntity,
) -> Result<(), Box<dyn Error>> {
    // conn.execute(
    //     "UPDATE domains SET
    //         expiration_date = ?1,
    //         registrar = ?2,
    //         status = ?3,
    //         updated_at = ?4
    //      WHERE id = ?5",
    //     params![
    //         domain.expiration_date,
    //         domain.registrar,
    //         domain.status.to_string(),
    //         Utc::now().to_string(),
    //         domain.id,
    //     ],
    // )?;
    Ok(())
}

/// 获取用户的所有域名
pub fn get_account_domains(
    conn: &DatabaseConnection,
    domain_id: Option<i64>,
) -> Result<Vec<RecordEntity>, Box<dyn Error>> {
    // let mut stmt = conn.prepare(
    //     "select id, account_id, domain_id, record_name, record_type, value, ttl, create_at from domain_records where domain_id = ?1;",
    // )?;

    // let domain_iter = stmt.query_map([domain_id], |row| {
    //     Ok(RecordEntity {
    //         id: row.get(0)?,
    //         account_id: row.get(1)?,
    //         domain_id: row.get(2)?,
    //         record_name: row.get(3)?,
    //         record_type: row.get(4)?,
    //         record_value: row.get(5)?,
    //         ttl: row.get(6)?,
    //     })
    // })?;
    // 
    // let mut records: Vec<RecordEntity> = Vec::new();
    // for domain in domain_iter {
    //     records.push(domain?);
    // }
    Ok(vec![])
}

/// 删除域名
pub fn delete_domain(conn: &DatabaseConnection, domain_id: i64) -> Result<(), Box<dyn Error>> {
    // conn.execute("DELETE FROM domains WHERE id = ?1", [domain_id])?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::gui::model::domain::DnsProvider;
//     use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
//     use crate::models::account::NewAccount;
//     use crate::models::domain::{DomainStatus, NewDomain};
//     use crate::storage::{add_domain,  init_memory_database};
//     use secrecy::{ExposeSecret, SecretString};
//
//     #[test]
//     fn it_works() {
//         let mut connection = init_memory_database().unwrap();
//
//         let _vault = String::from("stanic");
//
//         let password: SecretString = SecretString::from("12123");
//
//         let account = create_account(
//             &mut connection,
//             NewAccount {
//                 provider: DnsProvider::Aliyun,
//                 username: "stanic".to_string(),
//                 email: "example@qq.com".to_string(),
//                 credential: Credential::UsernamePassword(UsernamePasswordCredential {
//                     username: _vault.clone(),
//                     password: password.expose_secret().to_string(),
//                 }),
//                 master_key: Default::default(),
//                 api_keys: vec![],
//                 created_at: Utc::now().to_string(),
//             },
//         )
//         .expect("创建连接失败！");
//
//         let new_domain = NewDomain {
//             account_id: account.id,
//             domain_name: "".to_string(),
//             registration_date: None,
//             expiration_date: None,
//             registrar: None,
//             status: DomainStatus::Active,
//         };
//
//         let new_domain_entity = add_domain(&mut connection, new_domain).unwrap();
//
//         assert_eq!(account.username, "stanic", "变量名错误");
//
//         let record = add_record(
//             &mut connection,
//             NewRecord {
//                 account_id: account.id,
//                 domain_id: new_domain_entity.id,
//                 record_name: String::from("name"),
//                 record_type: String::from("@"),
//                 record_value: String::from("127.0.0.1"),
//                 ttl: 10,
//             },
//             &new_domain_entity,
//         )
//         .expect("账号创建失败");
//
//         assert_eq!(record.record_value, "127.0.0.1", "保存用户名异常");
//         assert_eq!(record.account_id, account.id, "保存用户名异常");
//
//         let domain_list = get_account_domains(&connection, Some(account.id)).expect("查询账号失败");
//         assert_eq!(domain_list.len(), 1, "获取账号失败");
//
//         let record = domain_list.get(0).unwrap();
//         assert_eq!(record.record_name, "name");
//         assert_eq!(record.record_value, "127.0.0.1");
//
//         let domain_list = get_account_domains(&connection, None).expect("查询账号失败");
//         assert_eq!(domain_list.len(), 0, "获取账号失败");
//     }
// }
