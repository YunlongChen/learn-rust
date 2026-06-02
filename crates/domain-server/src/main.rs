pub mod config;
pub mod database;
pub mod entities;
pub mod handlers;
pub mod logger;
pub mod migrations;

pub use database::*;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use tera::Tera;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
    tera: Tera,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    logger::init();
    info!("Starting server");

    let port = config::get().server.port();

    println!("Starting server at http://localhost:{}", port);

    let database_config = &config::get().database;

    // 建立数据库连接
    let db = establish_connection(database_config)
        .await
        .expect("获取数据库连接发生了异常!");

    // // 运行数据库迁移
    // Migrator::up(&db, None)
    //     .await
    //     .expect("Failed to run migrations");
    dbg!("连接状态", &db);

    // 初始化模板引擎
    let tera = async {
        match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    }
    .await;

    // 创建应用状态
    let app_state = AppState { db, tera };

    tracing::info!("Starting server at http://localhost:{}", port);
    dbg!("Starting server at http://localhost:{}", port);

    let result = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            // API 路由
            .service(handlers::provider::list_providers)
            .service(handlers::provider::create_provider)
            .service(handlers::domain::list_domains)
            .service(handlers::domain::create_domain)
            .service(handlers::dns_record::list_dns_records)
            .service(handlers::dns_record::create_dns_record)
            // 用户界面路由
            .service(handlers::ui::index)
            .service(handlers::ui::providers_page)
            .service(handlers::ui::domains_page)
            .service(handlers::ui::dns_records_page)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await;
    dbg!("Starting server at http://localhost:{}", port);
    result
}
