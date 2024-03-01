mod endpoints;
mod error;
mod constant_parse;
pub mod constants;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::routing::{get, post};
use axum::{middleware, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::error::Error;
use axum::extract::{DefaultBodyLimit, Request};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use crate::constants::BAD_DOT_ENV;

#[derive(Clone)]
pub struct AppState {
    postgres: Pool<Postgres>,
}

pub async fn sample_response_handler() -> String {
    "Home page example response".to_string()
}
pub async fn authed_sample_response_handler() -> String {
    "Authed home page example response".to_string()
}

async fn create_default_account(pool: &Pool<Postgres>) {
    let salt = SaltString::generate(&mut OsRng);
    let password = Argon2::default()
        .hash_password(&env::var("DEFAULT_ADMIN_PASSWORD").expect(BAD_DOT_ENV).as_bytes(), &salt)
        .unwrap();

    let _ = sqlx::query!(
        "INSERT INTO users (username, pass, token, tok_expire) VALUES ('Admin', $1, '', NOW())",
        password.to_string(),
    )
    .execute(pool)
    .await;
}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let pool = PgPoolOptions::new()
        .max_connections(
            (&env::var("MAX_DB_POOL_CONNECTIONS").expect(BAD_DOT_ENV))
                .parse()
                .unwrap(),
        )
        .connect(&env::var("DATABASE_URL").expect(BAD_DOT_ENV))
        .await?;

    create_default_account(&pool).await;

    let state = AppState { postgres: pool }; //TODO: Maybe add redis here for caching queries

    // Strictly for admins
    let auth = Router::new()
        .route("/faculty/:id/edit", get(endpoints::faculties::update_faculty_fe))
        .route("/faculty/:id/edit", post(endpoints::faculties::update_faculty))
        .route("/faculty/:id/delete", post(endpoints::faculties::delete_faculty))
        .route("/faculty/new", post(endpoints::faculties::create_faculty))
        .route("/faculty/new", get(endpoints::faculties::create_faculty_fe))
        .route("/classes/new", post(endpoints::classes::create_class))
        .route("/classes/:id/delete", post(endpoints::classes::delete_class))
        .route("/classes/:id/edit", post(endpoints::classes::update_class))
        .route("/classes/:id/edit", get(endpoints::classes::update_class_fe))
        .route("/classes/new", get(endpoints::classes::create_class_fe))
        .route("/check_auth", get(authed_sample_response_handler))
        .layer(middleware::from_fn_with_state(state.clone(), endpoints::auth::auth_middleware::<axum::body::Body>));

    let uploader = Router::new()
        .route("/classes/:id/upload", post(endpoints::classes_files::upload))
        .layer(DefaultBodyLimit::max(12 * 1024* 1024)); //12MB

    // For endpoints that have differences when the user is authed or the user isn't authed
    let auth_differences = Router::new()
        .route("/classes/:id", get(endpoints::classes::view_class_fe))
        .route("/faculties", get(endpoints::faculties::view_faculties_fe))
        .route("/classes", get(endpoints::classes::filter_fe))
        .layer(middleware::from_fn_with_state(state.clone(), endpoints::auth::permissive_middleware::<axum::body::Body>));

    // For endpoints that don't care if the user is authed or not
    let no_auth = Router::new()
        .route("/", get(sample_response_handler))
        .route("/login", get(endpoints::auth::login_fe))
        .route("/login", post(endpoints::auth::login_handler));

    let app = Router::new()
        .nest("/", auth)
        .nest("/", auth_differences)
        .nest("/", no_auth)
        .nest("/", uploader)
        .with_state(state)
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = TcpListener::bind(&env::var("BIND_ADDRESS").unwrap())
        .await
        .expect("Cannot start server");

    println!("DCT running.");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
