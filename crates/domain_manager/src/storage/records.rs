use crate::models::domain::{DomainEntity, DomainStatus, NewDomain};
use crate::models::record::{NewRecord, RecordEntity};
use crate::storage::{dns_record, DnsRecordDbEntity};
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use std::error::Error;
use tracing::{error, info};
use ActiveValue::Set;

/// 添加新域名
pub async fn add_record(
    conn: &DatabaseConnection,
    new_record: NewRecord,
) -> Result<RecordEntity, String> {
    let model = dns_record::ActiveModel {
        id: Default::default(),
        domain_id: Set(new_record.domain_id.clone()),
        record_type: Set(new_record.record_type.clone()),
        name: Set(new_record.record_name.clone()),
        value: Set(new_record.record_value.clone()),
        ttl: Set(new_record.ttl.clone()),
        created_at: Default::default(),
        updated_at: Default::default(),
        priority: Default::default(),
    };

    Ok(DnsRecordDbEntity::insert(model)
        .exec(conn)
        .await
        .map(|insert_result|
            // 创建记录
            RecordEntity {
                id: insert_result.last_insert_id,
                domain_id: new_record.domain_id,
                record_name: new_record.record_name,
                record_type: new_record.record_type,
                record_value: new_record.record_value,
                ttl: new_record.ttl,
            })
        .map_err(|err| {
            //
            error!("发布信息了:{:?}", err);
            "添加记录发生了异常！"
        })?)
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

/// 批量添加DNS记录
pub async fn add_records_many(
    conn: &DatabaseConnection,
    new_records: Vec<NewRecord>,
) -> Result<Vec<RecordEntity>, String> {
    if new_records.is_empty() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    
    for new_record in new_records {
        match add_record(conn, new_record).await {
            Ok(record) => results.push(record),
            Err(err) => {
                error!("批量添加DNS记录时出错: {}", err);
                return Err(format!("批量添加DNS记录失败: {}", err));
            }
        }
    }
    
    info!("成功批量添加 {} 条DNS记录", results.len());
    Ok(results)
}

/// 根据域名ID删除所有DNS记录
pub async fn delete_records_by_domain(
    conn: &DatabaseConnection,
    domain_id: i64,
) -> Result<u64, String> {
    let delete_result = DnsRecordDbEntity::delete_many()
        .filter(dns_record::Column::DomainId.eq(domain_id))
        .exec(conn)
        .await
        .map_err(|err| {
            error!("删除域名DNS记录失败: {:?}", err);
            format!("删除域名DNS记录失败: {}", err)
        })?;
    
    info!("成功删除域名ID {} 的 {} 条DNS记录", domain_id, delete_result.rows_affected);
    Ok(delete_result.rows_affected)
}

/// 获取用户的所有域名
pub async fn get_records_by_domain(
    conn: &DatabaseConnection,
    domain_id: Option<i64>,
) -> Result<Vec<RecordEntity>> {
    let select = DnsRecordDbEntity::find();

    let select = match domain_id {
        None => select,
        Some(domain_condition) => select.filter(dns_record::Column::DomainId.eq(domain_condition)),
    };

    let find_result = select.all(conn).await?;

    // 查询域名记录成功
    info!("查询成功了！：「{}」", find_result.len());
    Ok(find_result
        .into_iter()
        .map(|record| RecordEntity {
            id: record.id,
            domain_id: record.domain_id,
            record_name: record.name,
            record_type: record.record_type,
            record_value: record.value,
            ttl: record.ttl,
        })
        .collect())
}

/// 删除域名
pub fn delete_domain(conn: &DatabaseConnection, domain_id: i32) -> Result<(), Box<dyn Error>> {
    // conn.execute("DELETE FROM domains WHERE id = ?1", [domain_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::domain::DnsProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::models::account::NewAccount;
    use crate::storage::{add_domain, create_account, init_memory_database};
    use chrono::Utc;
    use secrecy::{ExposeSecret, SecretString};
    use serde_test::assert_de_tokens;
    use std::ops::Index;
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    pub async fn it_works() {
        let connection = init_memory_database().await.unwrap();

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

        let new_record = NewRecord {
            account_id: account.id,
            domain_id: domain.id,
            record_name: "www".to_string(),
            record_type: "A".to_string(),
            record_value: "127.0.0.1".to_string(),
            ttl: 100,
        };

        let record = add_record(&connection, new_record).await;
        assert!(record.is_ok());

        let record_entity = record.unwrap();

        assert_eq!(record_entity.domain_id, domain.id);
        assert_eq!(record_entity.record_value, "127.0.0.1");
        assert_eq!(record_entity.ttl, 100);

        let vec = get_records_by_domain(&connection, Some(domain.id))
            .await
            .unwrap();

        assert_eq!(vec.len(), 1);

        let vec = get_records_by_domain(&connection, None).await.unwrap();

        assert_eq!(vec.len(), 1);

        let vec = get_records_by_domain(&connection, Some(2)).await.unwrap();

        assert_eq!(vec.len(), 0);
    }
}
