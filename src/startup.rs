use crate::configuration::Settings;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::health_check::health_check;
use crate::routes::subscriptions::subscribe;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use secrecy::ExposeSecret;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub async fn build(configuration: Settings) -> Result<Server, Box<dyn std::error::Error>> {
    let address = build_address(&configuration);
    let sender = build_sender(&configuration);
    let timeout = build_timeout(&configuration);
    let email_client = build_email_client(&configuration, sender, timeout);
    let connection_pool = build_connection_pool(&configuration);
    let listener = build_listener(address);
    run(listener, connection_pool, email_client).await
}
pub async fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, Box<dyn std::error::Error>> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

fn build_address(configuration: &Settings) -> String {
    format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    )
}
fn build_email_client(
    configuration: &Settings,
    sender: SubscriberEmail,
    timeout: std::time::Duration,
) -> EmailClient {
    EmailClient::new(
        configuration.email_client.base_url.clone(),
        sender,
        configuration.email_client.authorization_token.clone(),
        timeout,
    )
}
fn build_timeout(configuration: &Settings) -> std::time::Duration {
    std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds)
}
fn build_sender(configuration: &Settings) -> SubscriberEmail {
    configuration
        .email_client
        .sender()
        .expect("Failed to parse sender email")
}
fn build_connection_pool(configuration: &Settings) -> PgPool {
    PgPoolOptions::new()
        .connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to database")
}
fn build_listener(address: String) -> TcpListener {
    TcpListener::bind(address).expect("Failed to bind address")
}
