use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match NewSubscriber::try_from(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&new_subscriber, connection).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = Box<dyn std::error::Error>;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { name, email })
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
