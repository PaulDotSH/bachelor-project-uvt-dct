use std::env;
use std::error::Error;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::{middleware, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::constants::*;

pub mod collect_with_capacity;
mod constant_parse;
pub mod constants;
mod endpoints;
mod error;

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
        .hash_password(
            &env::var("DEFAULT_ADMIN_PASSWORD")
                .expect(BAD_DOT_ENV)
                .as_bytes(),
            &salt,
        )
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
        .route(
            "/faculty/:id/edit",
            get(endpoints::faculties::update_faculty_fe),
        )
        .route(
            "/faculty/:id/edit",
            post(endpoints::faculties::update_faculty),
        )
        .route(
            "/faculty/:id/delete",
            post(endpoints::faculties::delete_faculty),
        )
        .route("/faculty/new", post(endpoints::faculties::create_faculty))
        .route("/faculty/new", get(endpoints::faculties::create_faculty_fe))
        .route("/classes/new", post(endpoints::classes::create_class))
        .route(
            "/classes/:id/delete",
            post(endpoints::classes::delete_class),
        )
        .route("/classes/:id/edit", post(endpoints::classes::update_class))
        .route(
            "/classes/:id/edit",
            get(endpoints::classes::update_class_fe),
        )
        .route("/classes/new", get(endpoints::classes::create_class_fe))
        .route("/files/:id/delete", post(endpoints::classes_files::delete))
        .route("/check_auth", get(authed_sample_response_handler))
        .route("/export-csv", get(endpoints::administration::export_csv))
        .route("/export-json", get(endpoints::administration::export_json))
        .route(
            "/move-choices",
            get(endpoints::administration::move_choices),
        ) // TODO: Make this post and with a ui
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::auth_middleware::<axum::body::Body>,
        ))
        .layer(TraceLayer::new_for_http());

    let uploader = Router::new()
        .route(
            "/classes/:id/upload",
            post(endpoints::classes_files::upload),
        )
        .layer(DefaultBodyLimit::max(MAX_CLASS_FILE_SIZE));

    // For endpoints that have differences when the user is authed or the user isn't authed
    let auth_differences = Router::new()
        .route("/classes/:id", get(endpoints::classes::view_class_fe))
        .route("/faculties", get(endpoints::faculties::view_faculties_fe))
        .route("/classes", get(endpoints::classes::filter_fe))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::permissive_middleware::<axum::body::Body>,
        ));

    // For endpoints that don't care if the user is authed or not
    let no_auth = Router::new()
        .route("/", get(sample_response_handler))
        .route("/admin/login", get(endpoints::auth::admin_login_fe))
        .route("/admin/login", post(endpoints::auth::admin_login_handler))
        .route("/login", get(endpoints::auth::student_login_fe))
        .route("/login", post(endpoints::auth::student_login_handler));

    let student_auth = Router::new()
        .route("/pick", get(endpoints::choices::pick_fe))
        .route("/pick", post(endpoints::choices::pick))
        .route("/student-auth", get(sample_response_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::student_middleware::<axum::body::Body>,
        ));

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
