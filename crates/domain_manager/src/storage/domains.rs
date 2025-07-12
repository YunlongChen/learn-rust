use crate::models::account::Account;
use crate::models::domain::{DomainEntity, DomainStatus, NewDomain};
use crate::storage::domain::Model;
use crate::storage::entities::domain;
use crate::storage::{entities, DomainDbEntity, DomainModel};
use anyhow::Context;
use entities::dns_record::Entity as DnsRecord;
use iced::futures::TryFutureExt;
use log::info;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use std::error::Error;
use tracing::error;

/// 添加新域名
pub async fn add_domain(
    conn: DatabaseConnection,
    new_domain: NewDomain,
) -> Result<DomainEntity, anyhow::Error> {
    let new_domain = domain::ActiveModel {
        id: Default::default(),
        name: Set(new_domain.domain_name),
        provider_id: Set(new_domain.account.id),
        status: Set("active".to_string()),
        created_at: Default::default(),
        updated_at: Default::default(),
    };

    Ok(new_domain
        .insert(&conn)
        .await
        .map_err(|err| {
            // 记录原始错误到日志
            error!("添加域名信息发生了异常!: {}", err);
            // 然后，我们返回一个不带原始错误的错误？或者保留？
            // 但是，我们仍然希望给上层提供完整的错误信息，所以不应该丢弃。
            // 所以，我们返回一个同样的错误，但是后面会添加上下文。
            err
        })
        .map(|model| {
            info!("数据插入成功");
            DomainEntity {
                id: model.id,
                account_id: model.provider_id,
                domain_name: model.name,
                registration_date: None,
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                created_at: model.created_at.to_string(),
                updated_at: model.updated_at,
            }
        })
        .context("新增域名操作失败")?)
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
pub async fn get_account_domains(
    conn: DatabaseConnection,
    account_id: Option<i32>,
) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    let domain_list = DomainDbEntity::find()
        .order_by_asc(domain::Column::Name)
        .filter(domain::Column::ProviderId.eq(account_id))
        .all(&conn)
        .map_err(|e| {
            error!("查询数据库报错了:异常：{}", e);
            e
        })
        .await?
        .into_iter()
        .map(|domain| DomainEntity {
            id: domain.id,
            account_id: domain.provider_id,
            domain_name: domain.name,
            registration_date: None,
            expiration_date: None,
            registrar: None,
            status: DomainStatus::Active,
            created_at: domain.created_at.to_string(),
            updated_at: domain.updated_at,
        })
        .collect();
    Ok(domain_list)
}

/// 获取用户的所有域名
pub async fn list_domains(conn: DatabaseConnection) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    let domain_list = DomainDbEntity::find()
        .order_by_asc(domain::Column::Name)
        .all(&conn)
        .map_err(|e| {
            error!("查询数据库报错了:异常：{}", e);
            e
        })
        .await?
        .into_iter()
        .map(|domain| DomainEntity {
            id: domain.id,
            account_id: domain.provider_id,
            domain_name: domain.name,
            registration_date: None,
            expiration_date: None,
            registrar: None,
            status: DomainStatus::Active,
            created_at: domain.created_at.to_string(),
            updated_at: domain.updated_at,
        })
        .collect();
    Ok(domain_list)
}

/// 获取即将过期的域名
pub fn get_expiring_domains(
    conn: &DatabaseConnection,
    days: i32,
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
pub fn delete_domain(conn: &DatabaseConnection, domain_id: i32) -> Result<(), Box<dyn Error>> {
    // conn.execute("DELETE FROM domains WHERE id = ?1", [domain_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dm_logger::init_logging;
    use crate::gui::model::domain::DnsProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::models::account::NewAccount;
    use crate::storage::{create_account, init_memory_database};
    use chrono::Utc;
    use secrecy::{ExposeSecret, SecretString};

    #[tokio::test]
    async fn it_works() {
        init_logging();

        let connection = init_memory_database().await.unwrap();

        dbg!("初始化数据库成功");

        let _vault = String::from("stanic");

        let password: SecretString = SecretString::from("12123");

        let account = create_account(
            connection.clone(),
            NewAccount {
                provider: DnsProvider::Aliyun,
                username: "stanic".to_string(),
                email: "example@qq.com".to_string(),
                credential: Credential::UsernamePassword(UsernamePasswordCredential {
                    username: _vault.clone(),
                    password: password.expose_secret().to_string(),
                }),
            },
        )
        .await
        .expect("查询数据库发生了异常".into());

        assert_eq!(account.username, "stanic", "变量名错误");

        let new_account = account.clone();

        let domain = add_domain(
            connection.clone(),
            NewDomain {
                domain_name: String::from("chenyunlong.cn"),
                registration_date: Some(Utc::now().to_string()),
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                account: new_account,
            },
        )
        .await
        .unwrap();

        assert_eq!(domain.domain_name, "chenyunlong.cn", "保存用户名异常");
        assert_eq!(domain.account_id, account.id, "保存用户名异常");
        assert_eq!(domain.updated_at, None);

        let domain_list = get_account_domains(connection.clone(), Some(account.id))
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        let domain_list = get_account_domains(connection.clone(), None)
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 0, "获取账号失败");

        let domain_list = list_domains(connection.clone())
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");
    }
}
