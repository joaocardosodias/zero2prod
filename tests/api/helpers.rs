use fake::faker::address;
use secrecy::ExposeSecret;
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use std::sync::LazyLock;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};

use zero2prod::telemetry::{get_subscriber, init_subscriber};

use zero2prod::email_client::EmailClient;
use zero2prod::startup::{Application, get_connection_pool};
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber = get_subscriber("test".into(), "info".into());
    init_subscriber(subscriber);
});
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Failed to parse sender email");
    let timeout = std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds);
    let email_client = EmailClient::new(
        configuration.email_client.base_url.clone(),
        sender_email,
        configuration.email_client.authorization_token.clone(),
        timeout,
    );
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build server");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());
    TestApp {
        address: address,
        db_pool: get_connection_pool(&configuration.database),
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "app".to_string(),
        password: Secret::new("secret".to_string()),
        ..config.clone()
    };
    let mut connection =
        PgConnection::connect(&maintenance_settings.connection_string().expose_secret())
            .await
            .expect("Failed to connect to database");
    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./scripts/migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations");
    connection_pool
}
