pub mod units;
mod chat;

use std::sync::{Mutex, Arc};
use dotenv::dotenv;
use sqlx::{
    Pool, Postgres,
    postgres::PgPoolOptions
};
use actix_web::{web, HttpServer, App};
use actix_cors::Cors;

pub struct AppState {
    pub db: Pool<Postgres>,
    pub chat_state: Arc<Mutex<chat::State>>
}

pub type AppStateData = web::Data<AppState>;

pub async fn serve() -> std::io::Result<()> {
    dotenv().ok();

    let chat_state = Arc::new(Mutex::new(chat::State::default()));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await
        .expect("error build a connection pool");

    let app_state = web::Data::new(
        AppState {
            db: db.clone(),
            chat_state: chat_state.clone()
        }
    );

    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS units (
                id SERIAL PRIMARY KEY,
                title VARCHAR(200),
                content TEXT
            );
        "#
    ).execute(&db)
    .await.unwrap();

    HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
        )
        .app_data(app_state.clone())
        .service(chat::service)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}