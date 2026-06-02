use crate::entities::domain::{self, Entity as Domain};
use actix_web::{get, post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateDomainForm {
    name: String,
    provider_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct DomainResponse {
    id: Uuid,
    name: String,
    provider_id: Uuid,
    status: String,
}

#[get("/domains")]
pub async fn list_domains(db: web::Data<DatabaseConnection>) -> impl Responder {
    let db = db.get_ref();
    let domains = Domain::find()
        .order_by_asc(domain::Column::Name)
        .all(db)
        .await
        .unwrap();

    let domains: Vec<DomainResponse> = domains
        .into_iter()
        .map(|d| DomainResponse {
            id: d.id,
            name: d.name,
            provider_id: d.provider_id,
            status: d.status,
        })
        .collect();

    HttpResponse::Ok().json(domains)
}

#[post("/domains")]
pub async fn create_domain(
    db: web::Data<DatabaseConnection>,
    form: web::Form<CreateDomainForm>,
) -> impl Responder {
    let db = db.get_ref();
    let new_domain = domain::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(form.name.clone()),
        provider_id: Set(form.provider_id),
        status: Set("active".to_string()),
        ..Default::default()
    };

    match new_domain.insert(db).await {
        Ok(domain) => HttpResponse::Created().json(DomainResponse {
            id: domain.id,
            name: domain.name,
            provider_id: domain.provider_id,
            status: domain.status,
        }),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error creating domain: {}", e)),
    }
}
