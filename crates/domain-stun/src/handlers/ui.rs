//! Web UI handlers (placeholder for template rendering)

use actix_web::{HttpResponse, web};
use tera::Tera;

pub async fn render_dashboard(tera: web::Data<Tera>) -> HttpResponse {
    let context = tera::Context::new();
    match tera.render("dashboard.html", &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body("<html><body><h1>Domain STUN Dashboard</h1></body></html>"),
    }
}

pub async fn render_agents(tera: web::Data<Tera>) -> HttpResponse {
    let context = tera::Context::new();
    match tera.render("agents.html", &context) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body("<html><body><h1>Agents</h1></body></html>"),
    }
}
