use crate::entities::provider::{self, Entity as Provider};
use actix_web::{get, post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateProviderForm {
    name: String,
    api_key: String,
    api_secret: String,
    extra_config: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProviderResponse {
    id: Uuid,
    name: String,
    api_key: String,
    api_secret: String,
    extra_config: Option<serde_json::Value>,
}

#[get("/providers")]
pub async fn list_providers(db: web::Data<DatabaseConnection>) -> impl Responder {
    let db = db.get_ref();
    let providers = Provider::find()
        .order_by_asc(provider::Column::Name)
        .all(db)
        .await
        .unwrap();

    let providers: Vec<ProviderResponse> = providers
        .into_iter()
        .map(|p| ProviderResponse {
            id: p.id,
            name: p.name,
            api_key: p.api_key,
            api_secret: p.api_secret,
            extra_config: p.extra_config,
        })
        .collect();

    HttpResponse::Ok().json(providers)
}

#[post("/providers")]
pub async fn create_provider(
    db: web::Data<DatabaseConnection>,
    form: web::Form<CreateProviderForm>,
) -> impl Responder {
    let db = db.get_ref();
    let new_provider = provider::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(form.name.clone()),
        api_key: Set(form.api_key.clone()),
        api_secret: Set(form.api_secret.clone()),
        extra_config: Set(form
            .extra_config
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())),
        ..Default::default()
    };

    match new_provider.insert(db).await {
        Ok(provider) => HttpResponse::Created().json(ProviderResponse {
            id: provider.id,
            name: provider.name,
            api_key: provider.api_key,
            api_secret: provider.api_secret,
            extra_config: provider.extra_config,
        }),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error creating provider: {}", e))
        }
    }
}
