use crate::configuration::DatabaseSettings;
use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::health_check::health_check;
use crate::routes::subscriptions::subscribe;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use secrecy::ExposeSecret;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

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
            .app_data(web::Data::new(connection_pool.clone()))
            .app_data(web::Data::new(email_client.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
pub fn get_connection_pool(configuratiom: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_lazy(configuratiom.connection_string().expose_secret())
        .expect("Failed to connect to database")
}

pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, Box<dyn std::error::Error>> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        
        let timeout =
            std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds);
        let sender = configuration
            .email_client
            .sender()
            .expect("Failed to parse sender email");
        let email_client = EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender,
            configuration.email_client.authorization_token.clone(),
            timeout,
        );
        let connection_pool = PgPoolOptions::new()
            .connect_lazy(&configuration.database.connection_string().expose_secret())
            .expect("Failed to connect to database");
        let listener = TcpListener::bind(address).expect("Failed to bind address");
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client).await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
