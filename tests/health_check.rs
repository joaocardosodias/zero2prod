use secrecy::ExposeSecret;
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use std::sync::LazyLock;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber = get_subscriber("test".into(), "info".into());
    init_subscriber(subscriber);
});
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
#[tokio::test]
async fn health_check_works() {
    LazyLock::force(&TRACING);
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", app_address.address))
        .send()
        .await
        .expect("Failed to send request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    let connection_pool = configure_database(&configuration.database).await;
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://localhost:{}", port),
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn subscrive_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app().await;
    let connection_pool = app_address.db_pool;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40example.com";
    let response = client
        .post(format!("{}/subscriptions", app_address.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&connection_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "ursula_le_guin@example.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscrive_returns_a_400_for_invalid_form_data() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("", "missing name and email"),
        ("email=ursula_le_guin%40example.com", "missing name"),
    ];
    for (invalid_body, _error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app_address.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request");
        assert_eq!(400, response.status().as_u16());
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
