use crate::models::domain::{DomainEntity, DomainStatus, NewDomain};
use crate::storage::entities;
use crate::storage::entities::domain;
use chrono::Utc;
use entities::dns_record::Entity as DnsRecord;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::error::Error;
use uuid::Uuid;

/// 添加新域名
pub async fn add_domain(
    conn: &DatabaseConnection,
    new_domain: NewDomain,
) -> Result<DomainEntity, Box<dyn Error>> {
    let now = Utc::now().to_string();

    let new_domain = domain::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Default::default(),
        provider_id: Default::default(),
        status: Set("active".to_string()),

        created_at: Default::default(),
        updated_at: Default::default(),
    };

    new_domain.insert(conn).await?;

    Ok(DomainEntity {
        id: 0,
        account_id: 0,
        domain_name: "".to_string(),
        registration_date: None,
        expiration_date: None,
        registrar: None,
        status: DomainStatus::Active,
        created_at: "".to_string(),
        updated_at: "".to_string(),
    })
}

/// 更新域名信息
pub async fn update_domain(
    conn: &DatabaseConnection,
    domain: &DnsRecord,
) -> Result<(), Box<dyn Error>> {
    // let pear: Option<DnsRecord::Model> = DnsRecord::find_by_id("28").one(conn).await?;
    //
    // // Into ActiveModel
    // let mut pear: DnsRecord::ActiveModel = pear.unwrap().into();
    //
    // // Update name attribute
    // pear.name = Set("Sweet pear".to_owned());
    //
    // // SQL: `UPDATE "fruit" SET "name" = 'Sweet pear' WHERE "id" = 28`
    // let pear: DnsRecord::Model = pear.update(conn).await?;

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
    account_id: Option<i64>,
) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    // let mut stmt = conn.prepare(
    //     "select id, account_id, domain_name, expire_ad, create_at from domains
    //      WHERE account_id = ?1",
    // )?;
    //
    // let domain_iter = stmt.query_map([account_id], |row| {
    //     Ok(DomainEntity {
    //         id: row.get(0)?,
    //         account_id: row.get(1)?,
    //         domain_name: row.get(2)?,
    //         registration_date: None,
    //         expiration_date: None,
    //         registrar: None,
    //         status: DomainStatus::Active,
    //         created_at: "".into(),
    //         updated_at: "".into(),
    //     })
    // })?;
    //
    // let mut domains = Vec::new();
    // for domain in domain_iter {
    //     domains.push(domain?);
    // }

    Ok(vec![])
}

/// 获取用户的所有域名
pub async  fn list_domains(conn: &DatabaseConnection) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    // let mut stmt =
    //     conn.prepare("select id, account_id, domain_name, expire_ad, create_at from domains")?;
    //
    // let domain_iter = stmt.query_map([], |row| {
    //     Ok(DomainEntity {
    //         id: row.get(0)?,
    //         account_id: row.get(1)?,
    //         domain_name: row.get(2)?,
    //         registration_date: None,
    //         expiration_date: None,
    //         registrar: None,
    //         status: DomainStatus::Active,
    //         created_at: "".into(),
    //         updated_at: "".into(),
    //     })
    // })?;
    //
    // let mut domains = Vec::new();
    // for domain in domain_iter {
    //     domains.push(domain?);
    // }

    // Ok(domains)
    Ok(vec![])
}

/// 获取即将过期的域名
pub fn get_expiring_domains(
    conn: &DatabaseConnection,
    days: i64,
) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    // let expiration_threshold = Utc::now() + chrono::Duration::days(days);
    //
    // let mut stmt = conn.prepare(
    //     "SELECT
    //         id,
    //         account_id,
    //         domain_name,
    //         registration_date,
    //         expiration_date,
    //         registrar,
    //         status,
    //         created_at,
    //         updated_at
    //      FROM domains
    //      WHERE expiration_date <= ?1
    //         AND status = 'active'",
    // )?;
    //
    // let domain_iter = stmt.query_map([expiration_threshold.to_string()], |row| {
    //     let status_str: String = row.get(6)?;
    //
    //     Ok(DomainEntity {
    //         id: row.get(0)?,
    //         account_id: row.get(1)?,
    //         domain_name: row.get(2)?,
    //         registration_date: row.get(3)?,
    //         expiration_date: row.get(4)?,
    //         registrar: row.get(5)?,
    //         status: DomainStatus::from_str(&status_str).unwrap_or(DomainStatus::Active),
    //         created_at: row.get(7)?,
    //         updated_at: row.get(8)?,
    //     })
    // })?;
    //
    // let mut domains = Vec::new();
    // for domain in domain_iter {
    //     domains.push(domain?);
    // }

    // Ok(domains)
    Ok(vec![])
}

/// 删除域名
pub fn delete_domain(conn: &DatabaseConnection, domain_id: i64) -> Result<(), Box<dyn Error>> {
    // conn.execute("DELETE FROM domains WHERE id = ?1", [domain_id])?;
    Ok(())
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::gui::model::domain::DnsProvider;
//     use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
//     use crate::models::account::NewAccount;
//     use crate::storage::{create_account, init_memory_database};
//     use secrecy::{ExposeSecret, SecretString};
//
//     #[test]
//      fn it_works() {
//         let mut connection = init_memory_database().await.unwrap();
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
//         assert_eq!(account.username, "stanic", "变量名错误");
//
//         let domain = add_domain(
//             &mut connection,
//             NewDomain {
//                 account_id: account.id,
//                 domain_name: String::from("chenyunlong.cn"),
//                 registration_date: Some(Utc::now().to_string()),
//                 expiration_date: None,
//                 registrar: None,
//                 status: DomainStatus::Active,
//             },
//         )
//         .expect("账号创建失败");
//
//         assert_eq!(domain.domain_name, "chenyunlong.cn", "保存用户名异常");
//         assert_eq!(domain.account_id, account.id, "保存用户名异常");
//
//         let domain_list = get_account_domains(&connection, Some(account.id)).expect("查询账号失败");
//         assert_eq!(domain_list.len(), 1, "获取账号失败");
//
//         let domain_list = get_account_domains(&connection, None).expect("查询账号失败");
//         assert_eq!(domain_list.len(), 0, "获取账号失败");
//
//         let domain_list = list_domains(&connection).expect("查询账号失败");
//         assert_eq!(domain_list.len(), 3, "获取账号失败");
//     }
// }
