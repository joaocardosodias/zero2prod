use secrecy::ExposeSecret;
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use std::sync::LazyLock;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};


pub static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber = get_subscriber("test".into(), "info".into());
    init_subscriber(subscriber);
});
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}


pub async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    let connection_pool = configure_database(&configuration.database).await;
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Failed to parse sender email");
    let email_client = EmailClient::new(
        configuration.email_client.base_url.clone(),
        sender_email,
        configuration.email_client.authorization_token.clone(),
        std::time::Duration::from_secs(configuration.email_client.timeout_milliseconds),
    );
    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://localhost:{}", port),
        db_pool: connection_pool,
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