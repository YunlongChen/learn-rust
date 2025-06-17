use crate::models::domain::{Domain, DomainStatus, NewDomain};
use chrono::Utc;
use rusqlite::{params, Connection, Result};

/// 添加新域名
pub fn add_domain(conn: &Connection, new_domain: NewDomain) -> Result<Domain> {
    let now = Utc::now().to_string();

    conn.execute(
        "INSERT INTO domains (
            account_id, 
            domain_name, 
            registration_date, 
            expiration_date, 
            registrar, 
            status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_domain.account_id,
            new_domain.domain_name,
            new_domain.registration_date,
            new_domain.expiration_date,
            new_domain.registrar,
            new_domain.status.to_string(),
        ],
    )?;

    let domain_id = conn.last_insert_rowid();

    Ok(Domain {
        id: domain_id,
        account_id: new_domain.account_id,
        domain_name: new_domain.domain_name,
        registration_date: new_domain.registration_date,
        expiration_date: new_domain.expiration_date,
        registrar: new_domain.registrar,
        status: new_domain.status,
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

/// 更新域名信息
pub fn update_domain(conn: &Connection, domain: &Domain) -> Result<()> {
    conn.execute(
        "UPDATE domains SET 
            expiration_date = ?1,
            registrar = ?2,
            status = ?3,
            updated_at = ?4
         WHERE id = ?5",
        params![
            domain.expiration_date,
            domain.registrar,
            domain.status.to_string(),
            Utc::now().to_string(),
            domain.id,
        ],
    )?;
    Ok(())
}

/// 获取用户的所有域名
pub fn get_account_domains(conn: &Connection, account_id: i64) -> Result<Vec<Domain>> {
    let mut stmt = conn.prepare(
        "SELECT 
            id, 
            domain_name, 
            registration_date, 
            expiration_date, 
            registrar, 
            status, 
            created_at, 
            updated_at 
         FROM domains 
         WHERE account_id = ?1",
    )?;

    let domain_iter = stmt.query_map([account_id], |row| {
        let status_str: String = row.get(5)?;

        Ok(Domain {
            id: row.get(0)?,
            account_id,
            domain_name: row.get(1)?,
            registration_date: row.get(2)?,
            expiration_date: row.get(3)?,
            registrar: row.get(4)?,
            status: DomainStatus::from_str(&status_str).unwrap_or(DomainStatus::Active),
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    let mut domains = Vec::new();
    for domain in domain_iter {
        domains.push(domain?);
    }

    Ok(domains)
}

/// 获取即将过期的域名
pub fn get_expiring_domains(conn: &Connection, days: i64) -> Result<Vec<Domain>> {
    let expiration_threshold = Utc::now() + chrono::Duration::days(days);

    let mut stmt = conn.prepare(
        "SELECT 
            id, 
            account_id,
            domain_name, 
            registration_date, 
            expiration_date, 
            registrar, 
            status, 
            created_at, 
            updated_at 
         FROM domains 
         WHERE expiration_date <= ?1 
            AND status = 'active'",
    )?;

    let domain_iter = stmt.query_map([expiration_threshold.to_string()], |row| {
        let status_str: String = row.get(6)?;

        Ok(Domain {
            id: row.get(0)?,
            account_id: row.get(1)?,
            domain_name: row.get(2)?,
            registration_date: row.get(3)?,
            expiration_date: row.get(4)?,
            registrar: row.get(5)?,
            status: DomainStatus::from_str(&status_str).unwrap_or(DomainStatus::Active),
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;

    let mut domains = Vec::new();
    for domain in domain_iter {
        domains.push(domain?);
    }

    Ok(domains)
}

/// 删除域名
pub fn delete_domain(conn: &Connection, domain_id: i64) -> Result<()> {
    conn.execute("DELETE FROM domains WHERE id = ?1", [domain_id])?;
    Ok(())
}
