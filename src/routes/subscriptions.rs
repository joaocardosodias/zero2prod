use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

pub async fn subscribe(_form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding subscriber",%request_id,subscriber_email=%_form.email,subscriber_name=%_form.name);
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving subscriber details in the database");
    match sqlx::query!(
        r#"INSERT INTO subscriptions (id,email,name,subscribed_at) VALUES ($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        _form.email,
        _form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
