use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use actix_web::{HttpResponse, web};

pub async fn subscribe(_form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id,email,name,subscribed_at) VALUES ($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        _form.email,
        _form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    .expect("Failed to  query");
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
