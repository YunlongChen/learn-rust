use crate::entities::dns_record;
use crate::entities::domain;
use crate::entities::provider;
use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use tera::{Context, Tera};

#[get("/")]
pub async fn index(db: web::Data<DatabaseConnection>, tera: web::Data<Tera>) -> impl Responder {
    let db = db.get_ref();
    let mut ctx = Context::new();

    // 获取服务商列表
    let providers = provider::Entity::find().all(db).await.unwrap_or_default();
    ctx.insert("providers", &providers);

    // 获取域名列表
    let domains = domain::Entity::find().all(db).await.unwrap_or_default();
    ctx.insert("domains", &domains);

    // 获取DNS记录列表
    let dns_records = dns_record::Entity::find().all(db).await.unwrap_or_default();
    ctx.insert("dns_records", &dns_records);

    let rendered = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/providers")]
pub async fn providers_page(
    db: web::Data<DatabaseConnection>,
    tera: web::Data<Tera>,
) -> impl Responder {
    let db = db.get_ref();
    let providers = provider::Entity::find().all(db).await.unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("providers", &providers);
    let rendered = tera.get_ref().render("providers.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/domains")]
pub async fn domains_page(
    db: web::Data<DatabaseConnection>,
    tera: web::Data<Tera>,
) -> impl Responder {
    let db = db.get_ref();
    let domains = domain::Entity::find().all(db).await.unwrap_or_default();

    let providers = provider::Entity::find().all(db).await.unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("domains", &domains);
    ctx.insert("providers", &providers);

    let rendered = tera.render("domains.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/dns_records")]
pub async fn dns_records_page(
    db: web::Data<DatabaseConnection>,
    tera: web::Data<Tera>,
) -> impl Responder {
    let db = db.get_ref();
    let dns_records = dns_record::Entity::find().all(db).await.unwrap_or_default();

    let domains = domain::Entity::find().all(db).await.unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("dns_records", &dns_records);
    ctx.insert("domains", &domains);

    let rendered = tera.render("dns_records.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
