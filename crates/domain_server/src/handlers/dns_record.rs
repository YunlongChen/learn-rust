use crate::entities::dns_record::{self, Entity as DnsRecord};
use actix_web::{get, post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateDnsRecordForm {
    domain_id: Uuid,
    record_type: String,
    name: String,
    value: String,
    ttl: i32,
    priority: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct DnsRecordResponse {
    id: Uuid,
    domain_id: Uuid,
    record_type: String,
    name: String,
    value: String,
    ttl: i32,
    priority: Option<i32>,
}

#[get("/dns_records")]
pub async fn list_dns_records(db: web::Data<DatabaseConnection>) -> impl Responder {
    let db = db.get_ref();
    let records = DnsRecord::find()
        .order_by_asc(dns_record::Column::Name)
        .all(db)
        .await
        .unwrap();

    let records: Vec<DnsRecordResponse> = records
        .into_iter()
        .map(|r| DnsRecordResponse {
            id: r.id,
            domain_id: r.domain_id,
            record_type: r.record_type,
            name: r.name,
            value: r.value,
            ttl: r.ttl,
            priority: r.priority,
        })
        .collect();

    HttpResponse::Ok().json(records)
}

#[post("/dns_records")]
pub async fn create_dns_record(
    db: web::Data<DatabaseConnection>,
    form: web::Form<CreateDnsRecordForm>,
) -> impl Responder {
    let db = db.get_ref();
    let new_record = dns_record::ActiveModel {
        id: Set(Uuid::new_v4()),
        domain_id: Set(form.domain_id),
        record_type: Set(form.record_type.clone()),
        name: Set(form.name.clone()),
        value: Set(form.value.clone()),
        ttl: Set(form.ttl),
        priority: Set(form.priority),
        ..Default::default()
    };

    match new_record.insert(db).await {
        Ok(record) => HttpResponse::Created().json(DnsRecordResponse {
            id: record.id,
            domain_id: record.domain_id,
            record_type: record.record_type,
            name: record.name,
            value: record.value,
            ttl: record.ttl,
            priority: record.priority,
        }),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error creating DNS record: {}", e))
        }
    }
}
