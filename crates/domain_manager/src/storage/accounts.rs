use crate::models::account::{Account, ApiKey, NewAccount};
use crate::storage::encryption::{encrypt_data, verify_password};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use std::error::Error;

/// 创建新账户
pub fn create_account(
    conn: &mut Connection,
    new_account: NewAccount,
) -> Result<Account, Box<dyn Error>> {
    // let (hashed_password, salt) = hash_password(&new_account.password.clone().into());
    let transaction = conn.transaction()?;

    transaction.execute(
        "INSERT INTO accounts (username, email, provider_type, credential_type, credential_data, salt,created_at)
         VALUES (?1, ?2, ?3, ?4,?5,?6, ?7)",
        params![
            new_account.username,
            new_account.email,
            new_account.provider.value(),
            new_account.credential.credential_type(),
            new_account.credential.raw_data(),
            "".to_string(),
            Utc::now().to_string()
        ],
    )?;

    let account_id = transaction.last_insert_rowid();

    // 创建API密钥（如果提供）
    let mut api_keys = Vec::new();
    for key in new_account.api_keys {
        let encrypted_key = encrypt_data(&key.key, &new_account.master_key)?;
        transaction.execute(
            "INSERT INTO api_keys (account_id, key_name, encrypted_key) 
             VALUES (?1, ?2, ?3)",
            params![account_id, key.key_name, encrypted_key.expose_secret()],
        )?;

        api_keys.push(ApiKey {
            id: transaction.last_insert_rowid(),
            key_name: key.key_name,
            key: key.key,
        });
    }

    transaction.commit()?;

    Ok(Account {
        id: account_id,
        username: new_account.username,
        email: new_account.email,
        credential_data: new_account.credential.raw_data(),
        salt: "salt".to_string(),
        api_keys,
        created_at: Utc::now().to_string(),
        last_login: None,
        credential_type: new_account.credential.credential_type(),
        provider_type: "".to_string(),
    })
}

/// 查询所有账户
pub fn list_accounts(conn: &Connection) -> Result<Vec<Account>, Box<dyn Error>> {
    let mut statement =
        conn.prepare("select id, username, email, provider_type, credential_type, credential_data, salt, created_at FROM accounts")?;

    let key_iter = statement.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            provider_type: row.get(3)?,
            credential_type: row.get(4)?,
            credential_data: row.get(5)?,
            salt: row.get(6)?,
            api_keys: vec![],
            created_at: row.get(7)?,
            last_login: None,
        })
    })?;

    let mut api_keys = Vec::new();
    for key in key_iter {
        api_keys.push(key?);
    }
    Ok(api_keys)
}

/// 验证用户登录
pub fn verify_login(
    conn: &Connection,
    username: &str,
    password: &SecretString,
) -> Result<Option<Account>, Box<dyn Error>> {
    let account: Option<(i64, String, String, String, String, Option<String>)> = conn
        .query_row(
            "SELECT id, email, credential_data, salt, created_at, last_login 
             FROM accounts WHERE username = ?1",
            [username],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()?;

    match account {
        Some((id, email, stored_hash, salt, created_at, last_login)) => {
            if verify_password(password, &stored_hash, &salt) {
                // 加载API密钥
                let api_keys = get_api_keys(conn, id)?;

                // 更新最后登录时间
                conn.execute(
                    "UPDATE accounts SET last_login = ?1 WHERE id = ?2",
                    params![Utc::now().to_string(), id],
                )?;

                Ok(Some(Account {
                    id,
                    provider_type: email.clone(),
                    username: username.to_string(),
                    email,
                    credential_type: "UsernamePassword".to_string(),
                    credential_data: stored_hash.clone(),
                    salt,
                    api_keys,
                    created_at,
                    last_login,
                }))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// 获取账户的API密钥
fn get_api_keys(conn: &Connection, account_id: i64) -> Result<Vec<ApiKey>, Box<dyn Error>> {
    let mut stmt =
        conn.prepare("SELECT id, key_name, encrypted_key FROM api_keys WHERE account_id = ?1")?;

    let key_iter = stmt.query_map([account_id], |row| {
        Ok((row.get(0)?, row.get(1)?, SecretBox::new(row.get(2)?)))
    })?;

    let mut api_keys = Vec::new();
    for key in key_iter {
        let (id, key_name, encrypted_key) = key?;
        // 在实际应用中，这里需要主密钥来解密
        // 暂时返回加密数据，解密应在业务层进行
        api_keys.push(ApiKey {
            id,
            key_name,
            key: encrypted_key,
        });
    }

    Ok(api_keys)
}

/// 更新账户信息
pub fn update_account(conn: &Connection, account: &Account) -> Result<()> {
    conn.execute(
        "UPDATE accounts SET email = ?1, last_login = ?2 WHERE id = ?3",
        params![account.email, account.last_login, account.id],
    )?;
    Ok(())
}

/// 添加API密钥
pub fn add_api_key(
    conn: &Connection,
    account_id: i64,
    key_name: &str,
    key_value: &SecretString,
    master_key: &SecretString,
) -> Result<ApiKey, Box<dyn Error>> {
    let encrypted_key = encrypt_data(key_value, master_key)?;

    conn.execute(
        "INSERT INTO api_keys (account_id, key_name, encrypted_key) 
         VALUES (?1, ?2, ?3)",
        params![account_id, key_name, encrypted_key.expose_secret()],
    )?;

    Ok(ApiKey {
        id: conn.last_insert_rowid(),
        key_name: key_name.to_string(),
        key: SecretString::from(key_value.clone()),
    })
}

/// 删除账户
pub fn delete_account(conn: &Connection, account_id: i64) -> Result<()> {
    // 外键设置为CASCADE，会自动删除关联域名和API密钥
    conn.execute("DELETE FROM accounts WHERE id = ?1", [account_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::domain::DnsProvider;
    use crate::gui::types::credential::{Credential, UsernamePasswordCredential};
    use crate::storage::init_memory_database;

    #[test]
    fn it_works() {
        let mut connection = init_memory_database().unwrap();

        let _vault = String::from("stanic");

        let password: SecretString = SecretString::from("12123");

        let api_key = create_account(
            &mut connection,
            NewAccount {
                provider: DnsProvider::Aliyun,
                username: "stanic".to_string(),
                email: "example@qq.com".to_string(),
                credential: Credential::UsernamePassword(UsernamePasswordCredential {
                    username: _vault.clone(),
                    password: password.expose_secret().to_string(),
                }),
                master_key: Default::default(),
                api_keys: vec![],
                created_at: Utc::now().to_string(),
            },
        )
        .expect("创建连接失败！");

        assert_eq!(api_key.username, "stanic", "变量名错误");

        let accounts_result = list_accounts(&connection);
        dbg!("{?}", &accounts_result);

        let accounts = accounts_result.unwrap();

        assert_eq!(accounts.len(), 2);

        let account = accounts.get(1).take();

        match account {
            None => {}
            Some(account) => {
                let credential: Credential = account.clone().try_into().unwrap();
                assert_eq!("Aliyun", account.provider_type, "变量名错误");

                if let Credential::UsernamePassword(credential) = credential {
                    assert_eq!("stanic", credential.username, "变量名错误");
                    assert_eq!("12123", credential.password, "变量名错误");
                }
            }
        }
    }
}
