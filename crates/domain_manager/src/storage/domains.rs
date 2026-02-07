use crate::models::domain::{DomainEntity, DomainStatus, NewDomain};
use crate::storage::entities::domain;
use crate::storage::{entities, DomainDbEntity};
use anyhow::Context;
use entities::dns_record::Entity as DnsRecord;
use iced::futures::TryFutureExt;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use std::error::Error;
use tracing::{error, info};

/// 添加新域名
pub async fn add_domain(
    conn: &DatabaseConnection,
    new_domain: NewDomain,
) -> Result<DomainEntity, anyhow::Error> {
    let new_domain = domain::ActiveModel {
        id: Default::default(),
        name: Set(new_domain.domain_name),
        provider_id: Set(new_domain.account_id),
        status: Set(new_domain.status.to_string().into()),
        created_at: Default::default(),
        updated_at: Default::default(),
    };

    Ok(new_domain
        .insert(conn)
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

/// 添加新域名
pub async fn add_domain_many(
    conn: &DatabaseConnection,
    new_domain_list: Vec<NewDomain>,
) -> Result<(), anyhow::Error> {
    // 处理空列表情况
    if new_domain_list.is_empty() {
        return Ok(());
    }

    let domain_entity_list: Vec<domain::ActiveModel> = new_domain_list
        .into_iter()
        .map(|domain| domain::ActiveModel {
            id: Default::default(),
            name: Set(domain.domain_name),
            provider_id: Set(domain.account_id),
            status: Set(String::from(domain.status.to_string())),
            created_at: Default::default(),
            updated_at: Default::default(),
        })
        .collect();

    let _result = domain::Entity::insert_many(domain_entity_list)
        .exec(conn)
        .await
        .map_err(|err| {
            // 记录原始错误到日志
            error!("数据库添加域名信息发生了异常!: {:?}", err);
            // 然后，我们返回一个不带原始错误的错误？或者保留？
            // 但是，我们仍然希望给上层提供完整的错误信息，所以不应该丢弃。
            // 所以，我们返回一个同样的错误，但是后面会添加上下文。
            err
        })
        .context("批量添加域名失败")?;
    Ok(())
}

/// 更新域名信息
pub async fn update_domain(
    _conn: &DatabaseConnection,
    _domain: &DnsRecord,
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
    conn: &DatabaseConnection,
    account_id: Option<i64>,
) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    let account_log_info = match account_id {
        None => "",
        Some(account) => &account.to_string(),
    };
    let select = DomainDbEntity::find().order_by_asc(domain::Column::Name);
    let select = match account_id {
        None => {
            info!("查询所有域名列表！");
            select
        }
        Some(account) => {
            info!("根据账号查询域名列表，账号标识：「{}」", account);
            select.filter(domain::Column::ProviderId.eq(account))
        }
    };
    let domain_list: Vec<DomainEntity> = select
        .all(conn)
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
    info!(
        "查询域名信息成功，账号表示：「{}」，查询结果数量：「{}」",
        account_log_info,
        &domain_list.len()
    );
    Ok(domain_list)
}

/// 获取用户的所有域名
pub async fn list_domains(conn: &DatabaseConnection) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    let domain_list = DomainDbEntity::find()
        .order_by_asc(domain::Column::Name)
        .all(conn)
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

/// 根据域名名称查找域名
pub async fn find_domain_by_name(
    conn: &DatabaseConnection,
    domain_name: &str,
) -> Result<Option<DomainEntity>, Box<dyn Error>> {
    let domain_model = DomainDbEntity::find()
        .filter(domain::Column::Name.eq(domain_name))
        .one(conn)
        .await
        .map_err(|e| {
            error!("根据域名名称查找域名失败: {}", e);
            Box::new(e) as Box<dyn Error>
        })?;

    match domain_model {
        Some(domain) => Ok(Some(DomainEntity {
            id: domain.id,
            account_id: domain.provider_id,
            domain_name: domain.name,
            registration_date: None,
            expiration_date: None,
            registrar: None,
            status: DomainStatus::Active,
            created_at: domain.created_at.to_string(),
            updated_at: domain.updated_at,
        })),
        None => Ok(None),
    }
}

/// 根据ID查找域名
pub async fn find_domain_by_id(
    conn: &DatabaseConnection,
    domain_id: i64,
) -> Result<Option<DomainEntity>, Box<dyn Error>> {
    let domain_model = DomainDbEntity::find_by_id(domain_id)
        .one(conn)
        .await
        .map_err(|e| {
            error!("根据ID查找域名失败: {}", e);
            Box::new(e) as Box<dyn Error>
        })?;

    match domain_model {
        Some(domain) => Ok(Some(DomainEntity {
            id: domain.id,
            account_id: domain.provider_id,
            domain_name: domain.name,
            registration_date: None,
            expiration_date: None,
            registrar: None,
            status: DomainStatus::Active,
            created_at: domain.created_at.to_string(),
            updated_at: domain.updated_at,
        })),
        None => Ok(None),
    }
}

/// 根据域名名称和账户ID查找域名
pub async fn find_domain_by_name_and_account(
    conn: &DatabaseConnection,
    domain_name: &str,
    account_id: i64,
) -> Result<Option<DomainEntity>, Box<dyn Error>> {
    let domain_model = DomainDbEntity::find()
        .filter(domain::Column::Name.eq(domain_name))
        .filter(domain::Column::ProviderId.eq(account_id))
        .one(conn)
        .await
        .map_err(|e| {
            error!("根据域名名称查找域名失败: {}", e);
            e
        })?;

    match domain_model {
        Some(domain) => {
            info!("找到域名: {} (ID: {})", domain_name, domain.id);
            Ok(Some(DomainEntity {
                id: domain.id,
                account_id: domain.provider_id,
                domain_name: domain.name,
                registration_date: None,
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                created_at: domain.created_at.to_string(),
                updated_at: domain.updated_at,
            }))
        }
        None => {
            info!("未找到域名: {}", domain_name);
            Ok(None)
        }
    }
}

/// 获取用户的所有域名
pub async fn count_all_domains(conn: &DatabaseConnection) -> Result<u64, Box<dyn Error>> {
    let count_result = DomainDbEntity::find()
        .count(conn)
        .map_err(|e| {
            error!("查询所有域名数量发生了异常：{:?}", e);
            e
        })
        .await?;
    Ok(count_result)
}

/// 获取即将过期的域名
pub fn get_expiring_domains(
    _conn: &DatabaseConnection,
    _days: i32,
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
pub async fn delete_domain(conn: &DatabaseConnection, domain_id: i64) -> Result<(), Box<dyn Error>> {
    use crate::storage::entities::domain;
    use sea_orm::EntityTrait;

    domain::Entity::delete_by_id(domain_id)
        .exec(conn)
        .await
        .map_err(|e| {
            error!("删除域名失败: {:?}", e);
            Box::new(e) as Box<dyn Error>
        })?;

    Ok(())
}

/// 根据账号ID获取域名列表
pub async fn list_domains_by_account(
    conn: &DatabaseConnection,
    account_id: i64,
) -> Result<Vec<DomainEntity>, Box<dyn Error>> {
    let domain_list = DomainDbEntity::find()
        .filter(domain::Column::ProviderId.eq(account_id))
        .order_by_asc(domain::Column::Name)
        .all(conn)
        .await
        .map_err(|e| {
            error!("查询域名失败: {}", e);
            Box::new(e) as Box<dyn Error>
        })?
        .into_iter()
        .map(|domain| DomainEntity {
            id: domain.id,
            account_id: domain.provider_id,
            domain_name: domain.name,
            registration_date: None,
            expiration_date: None,
            registrar: None,
            status: DomainStatus::Active, // TODO: 从数据库或状态获取
            created_at: domain.created_at.to_string(),
            updated_at: domain.updated_at,
        })
        .collect();
    Ok(domain_list)
}

/// 根据账号删除域名
pub async fn delete_domain_by_account(
    conn: &DatabaseConnection,
    account_id: i64,
) -> Result<(), Box<dyn Error + Send>> {
    DomainDbEntity::delete_many()
        .filter(domain::Column::ProviderId.eq(account_id))
        .exec(conn)
        .await
        .expect("更新发生了异常！");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::domain::DnsProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::models::account::NewAccount;
    use crate::storage::{create_account, init_memory_database};
    use crate::tests::test_utils::init_test_env;
    use chrono::Utc;
    use secrecy::{ExposeSecret, SecretString};

    #[tokio::test]
    pub async fn it_works() {
        init_test_env();
        let connection = init_memory_database().await.unwrap();

        let _vault = String::from("stanic");

        let password: SecretString = SecretString::from("12123");

        let account = create_account(
            &connection,
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
            &connection,
            NewDomain {
                domain_name: String::from("chenyunlong.cn"),
                registration_date: Some(Utc::now().to_string()),
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                account_id: new_account.id,
            },
        )
        .await
        .unwrap();

        assert_eq!(domain.domain_name, "chenyunlong.cn", "保存用户名异常");
        assert_eq!(domain.account_id, account.id, "保存用户名异常");
        assert_eq!(domain.updated_at, None);

        let domain_list = get_account_domains(&connection, Some(account.id))
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        let domain_list = get_account_domains(&connection, None)
            .await
            .expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        let domain_list = list_domains(&connection).await.expect("查询账号失败");
        assert_eq!(domain_list.len(), 1, "获取账号失败");

        add_domain_many(
            &connection,
            vec![NewDomain {
                domain_name: String::from("stanic.xyz"),
                registration_date: Some(Utc::now().to_string()),
                expiration_date: None,
                registrar: None,
                status: DomainStatus::Active,
                account_id: new_account.id,
            }],
        )
        .await
        .unwrap();

        let domain_list = get_account_domains(&connection, Some(account.id))
            .await
            .expect("查询域名失败");
        assert_eq!(domain_list.len(), 2, "获取账号失败");

        let result = domain_list
            .iter()
            .find(|domain| domain.domain_name == "stanic.xyz")
            .unwrap();
        assert_eq!(result.status, DomainStatus::Active);
        assert_eq!(result.account_id, account.id);

        let domains_count = count_all_domains(&connection).await.unwrap();
        assert_eq!(domains_count, 2, "查询域名数量异常，数量对不上！");
    }
}
