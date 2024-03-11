use std::env;
use std::error::Error;

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::{middleware, Router};
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Request};
use axum::routing::{get, post};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::net::unix::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::constants::BAD_DOT_ENV;

mod endpoints;
mod error;
mod constant_parse;
pub mod constants;
pub mod collect_with_capacity;

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

#[instrument]
async fn log_request_info(request: &Request<Body>, addr: &SocketAddr) {
    let headers = request.headers();
    let foo = format!("{:?}", addr);
    let ip = if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        forwarded_for.to_str().unwrap_or_else(|_| "Unknown")
    } else if let Some(real_ip) = headers.get("X-Real-IP") {
        real_ip.to_str().unwrap_or_else(|_| "Unknown")
    } else {
        foo.as_str()
        // addr.ip().to_string().as_str()
    };

    info!("IP: {}, Headers: {:?}", ip, headers);
}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    // Added tracing, more advanced tracing should be done in nginx or whatever alternative used
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

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
    let admin_auth = Router::new()
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
        .layer(middleware::from_fn_with_state(state.clone(), endpoints::auth::auth_middleware::<axum::body::Body>))
        .layer(TraceLayer::new_for_http());

    let uploader = Router::new()
        .route("/classes/:id/upload", post(endpoints::classes_files::upload))
        .layer(DefaultBodyLimit::max(12 * 1024 * 1024)); //12MB

    // For endpoints that have differences when the user is authed or the user isn't authed
    let auth_differences = Router::new()
        .route("/classes/:id", get(endpoints::classes::view_class_fe))
        .route("/faculties", get(endpoints::faculties::view_faculties_fe))
        .route("/classes", get(endpoints::classes::filter_fe))
        .layer(middleware::from_fn_with_state(state.clone(), endpoints::auth::permissive_middleware::<axum::body::Body>));

    // For endpoints that don't care if the user is authed or not
    let no_auth = Router::new()
        .route("/", get(sample_response_handler))
        .route("/admin/login", get(endpoints::auth::admin_login_fe))
        .route("/admin/login", post(endpoints::auth::admin_login_handler))
        .route("/login", get(endpoints::auth::student_login_fe))
        .route("/login", post(endpoints::auth::student_login_handler));


    let student_auth = Router::new()
        .route("/pick", get(endpoints::choices::pick_fe))
        .route("/student-auth", get(sample_response_handler))
        .layer(middleware::from_fn_with_state(state.clone(), endpoints::auth::student_middleware::<axum::body::Body>));

    let app = Router::new()
        .nest("/", admin_auth)
        .nest("/", auth_differences)
        .nest("/", student_auth)
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
