use crate::models::account::{Account, ApiKey, NewAccount};
use crate::storage::encryption::encrypt_data;
use crate::storage::{entities, AccountActiveModel, AccountEntity};
use iced::futures::TryFutureExt;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryOrder};
use secrecy::SecretString;
use std::error::Error;
use tracing::{error, info};

/// 创建新账户
pub async fn create_account(
    conn: &DatabaseConnection,
    new_account: NewAccount,
) -> Result<Account, String> {
    let id = Default::default();

    let active_model = AccountActiveModel {
        id,
        name: ActiveValue::Set(new_account.username),
        salt: ActiveValue::Set("123123".into()),
        last_login: Default::default(),
        credential_type: ActiveValue::Set(new_account.credential.credential_type()),
        credential_data: ActiveValue::Set(new_account.credential.raw_data().into()),
        provider_type: ActiveValue::Set(new_account.provider.value().into()),
        created_at: Default::default(),
        updated_at: Default::default(),
    };

    let result = active_model
        .insert(conn)
        .await
        .map_err(|err| error!("添加账号发生了异常:{:?}", err));

    match result {
        Ok(model) => {
            info!("添加账号成功,账号标识：{:?}", &model.id);
            Ok(Account {
                id: model.id,
                username: model.name.clone(),
                email: model.name.clone(),
                credential_data: model.credential_data,
                salt: "salt".to_string(),
                api_keys: vec![],
                created_at: model.created_at.to_string(),
                last_login: None,
                credential_type: new_account.credential.credential_type(),
                provider_type: "".to_string(),
            })
        }
        Err(err) => {
            error!("创建账号发生了异常,{:?}", err);
            Err(format!("查询数据库异常:{:?}", err))
        }
    }
}

/// 查询所有账户
pub async fn list_accounts(
    conn: &DatabaseConnection,
) -> Result<Vec<Account>, Box<dyn Error + Send>> {
    let accounts = AccountEntity::find()
        .order_by_asc(entities::account::Column::Name)
        .all(conn)
        .map_err(|e| {
            error!("查询数据库报错了:异常：{}", e);
            Box::new(e) as Box<dyn Error + Send>
        })
        .await?;

    info!("查询到的账号列表:{}", &accounts.len());

    let account_list: Vec<Account> = accounts
        .into_iter()
        .map(|account| Account {
            id: account.id,
            username: account.name,
            email: "".to_string(),
            salt: "".to_string(),
            api_keys: vec![],
            created_at: "".to_string(),
            last_login: account.last_login.map(|date| date.to_string()),
            credential_type: account.credential_type,
            credential_data: account.credential_data,
            provider_type: account.provider_type,
        })
        .collect();

    Ok(account_list)

    // let mut statement =
    //     conn.prepare("select id, username, email, provider_type, credential_type, credential_data, salt, created_at FROM accounts")?;
    //
    // let key_iter = statement.query_map([], |row| {
    //     Ok(Account {
    //         id: row.get(0)?,
    //         username: row.get(1)?,
    //         email: row.get(2)?,
    //         provider_type: row.get(3)?,
    //         credential_type: row.get(4)?,
    //         credential_data: row.get(5)?,
    //         salt: row.get(6)?,
    //         api_keys: vec![],
    //         created_at: row.get(7)?,
    //         last_login: None,
    //     })
    // })?;
    //
    // let mut api_keys = Vec::new();
    // for key in key_iter {
    //     api_keys.push(key?);
    // }
    // Ok(api_keys)
    // Ok(vec![])
}

/// 根据ID获取账户
pub async fn get_account_by_id(
    conn: &DatabaseConnection,
    id: i64,
) -> Result<Option<Account>, Box<dyn Error + Send>> {
    let account = AccountEntity::find_by_id(id).one(conn).await.map_err(|e| {
        error!("根据ID查询账户失败: {}", e);
        Box::new(e) as Box<dyn Error + Send>
    })?;

    Ok(account.map(|model| Account {
        id: model.id,
        username: model.name,
        email: "".to_string(), // 数据库中没有 email 字段
        salt: "".to_string(),
        api_keys: vec![],
        created_at: model.created_at.to_string(),
        last_login: model.last_login.map(|d| d.to_string()),
        credential_type: model.credential_type,
        credential_data: model.credential_data,
        provider_type: model.provider_type,
    }))
}

/// 验证用户登录
pub fn verify_login(
    _conn: &DatabaseConnection,
    _username: &str,
    _password: &SecretString,
) -> Result<Option<Account>, Box<dyn Error>> {
    // let account: Option<(i32, String, String, String, String, Option<String>)> = conn
    //     .query_row(
    //         "SELECT id, email, credential_data, salt, created_at, last_login
    //          FROM accounts WHERE username = ?1",
    //         [username],
    //         |row| {
    //             Ok((
    //                 row.get(0)?,
    //                 row.get(1)?,
    //                 row.get(2)?,
    //                 row.get(3)?,
    //                 row.get(4)?,
    //                 row.get(5)?,
    //             ))
    //         },
    //     )
    //     .optional()?;

    // match account {
    //     Some((id, email, stored_hash, salt, created_at, last_login)) => {
    //         if verify_password(password, &stored_hash, &salt) {
    //             // 加载API密钥
    //             let api_keys = get_api_keys(conn, id)?;
    //
    //             // 更新最后登录时间
    //             conn.execute(
    //                 "UPDATE accounts SET last_login = ?1 WHERE id = ?2",
    //                 params![Utc::now().to_string(), id],
    //             )?;
    //
    //             Ok(Some(Account {
    //                 id,
    //                 provider_type: email.clone(),
    //                 username: username.to_string(),
    //                 email,
    //                 credential_type: "UsernamePassword".to_string(),
    //                 credential_data: stored_hash.clone(),
    //                 salt,
    //                 api_keys,
    //                 created_at,
    //                 last_login,
    //             }))
    //         } else {
    //             Ok(None)
    //         }
    //     }
    //     None => Ok(None),
    // }
    Ok(None)
}

/// 获取账户的API密钥
fn get_api_keys(
    _conn: &DatabaseConnection,
    _account_id: i32,
) -> Result<Vec<ApiKey>, Box<dyn Error>> {
    // let mut stmt =
    //     conn.prepare("SELECT id, key_name, encrypted_key FROM api_keys WHERE account_id = ?1")?;
    //
    // let key_iter = stmt.query_map([account_id], |row| {
    //     Ok((row.get(0)?, row.get(1)?, SecretBox::new(row.get(2)?)))
    // })?;
    //
    // let mut api_keys = Vec::new();
    // for key in key_iter {
    //     let (id, key_name, encrypted_key) = key?;
    //     // 在实际应用中，这里需要主密钥来解密
    //     // 暂时返回加密数据，解密应在业务层进行
    //     api_keys.push(ApiKey {
    //         id,
    //         key_name,
    //         key: encrypted_key,
    //     });
    // }
    //
    // Ok(api_keys)
    Ok(vec![])
}

/// 更新账户信息
pub async fn update_account(
    conn: &DatabaseConnection,
    account: &Account,
) -> Result<(), Box<dyn Error>> {
    use crate::storage::entities::account;
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let active_model = account::ActiveModel {
        id: ActiveValue::Set(account.id),
        name: ActiveValue::Set(account.username.clone()),
        provider_type: ActiveValue::Set(account.provider_type.clone()),
        credential_data: ActiveValue::Set(account.credential_data.clone()),
        credential_type: ActiveValue::Set(account.credential_type.clone()),
        updated_at: ActiveValue::Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    active_model.update(conn).await.map_err(|e| {
        error!("更新账户失败: {:?}", e.to_string());
        Box::new(e) as Box<dyn Error>
    })?;

    Ok(())
}

/// 添加API密钥
pub fn add_api_key(
    _conn: &DatabaseConnection,
    _account_id: i32,
    _key_name: &str,
    key_value: &SecretString,
    master_key: &SecretString,
) -> Result<ApiKey, Box<dyn Error>> {
    let _encrypted_key = encrypt_data(key_value, master_key)?;

    // conn.execute(
    //     "INSERT INTO api_keys (account_id, key_name, encrypted_key)
    //      VALUES (?1, ?2, ?3)",
    //     params![account_id, key_name, encrypted_key.expose_secret()],
    // )?;
    //
    // Ok(ApiKey {
    //     id: conn.last_insert_rowid(),
    //     key_name: key_name.to_string(),
    //     key: SecretString::from(key_value.clone()),
    // })
    Ok(ApiKey {
        // id: conn.last_insert_rowid(),
        // key_name: key_name.to_string(),
        // key: SecretString::from(key_value.clone()),
        id: 0,
        key_name: "".to_string(),
        key: Default::default(),
    })
}

/// 删除账户
pub async fn delete_account(conn: &DatabaseConnection, account_id: i64) -> Result<(), Box<dyn Error>> {
    use crate::storage::entities::account;
    use sea_orm::EntityTrait;

    account::Entity::delete_by_id(account_id)
        .exec(conn)
        .await
        .map_err(|e| {
            error!("删除账户失败: {:?}", e);
            Box::new(e) as Box<dyn Error>
        })?;

    info!("账户删除成功: {}", account_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::domain::DnsProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::storage::init_memory_database;
    use crate::tests::test_utils::init_test_env;
    use secrecy::ExposeSecret;

    #[tokio::test]
    async fn it_works() {
        init_test_env();
        let connection = init_memory_database().await.unwrap();
        connection.ping().await.unwrap();

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
        let _vault = String::from("stanic");

        let _password: SecretString = SecretString::from("12123");

        let accounts_result = list_accounts(&connection);
        info!(target: "config", "正在加载配置文件:{}","ok");

        let accounts = accounts_result.await.unwrap();

        assert_eq!(accounts.len(), 1);

        let account = accounts.get(0).take().unwrap();

        let credential: Credential = account.clone().try_into().unwrap();
        assert_eq!("Aliyun", account.provider_type);
        assert_eq!("stanic", account.username);

        if let Credential::UsernamePassword(credential) = credential {
            assert_eq!("stanic", credential.username, "变量名错误");
            assert_eq!("12123", credential.password, "变量名错误");
        }
    }
}
