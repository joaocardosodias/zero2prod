use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;
use crate::domain::{NewSubscriber,SubscriberEmail,SubscriberName};

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    
    let new_subscriber=NewSubscriber{
        name:match SubscriberName::parse(form.name.clone()){
            Ok(name)=>name,
            Err(_)=>return HttpResponse::BadRequest().finish(),
        },
        email:match SubscriberEmail::parse(form.email.clone()){
            Ok(email)=>email,
            Err(_)=>return HttpResponse::BadRequest().finish(),
        },
    };
    match insert_subscriber(&new_subscriber, connection).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    connection: web::Data<PgPool>,
) -> Result<(), sqlx::Error> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding subscriber",%request_id,subscriber_email=%new_subscriber.email.as_ref(),subscriber_name=%new_subscriber.name.as_ref());
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving subscriber details in the database");
    sqlx::query!(
        r#"INSERT INTO subscriptions (id,email,name,subscribed_at) VALUES ($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
