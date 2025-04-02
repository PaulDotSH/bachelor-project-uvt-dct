use std::error::Error;
use std::{env, fs};

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::{middleware, Router};
use const_format::formatcp;
use lib::init_db;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use crate::constants::*;
pub mod collect_with_capacity;
pub mod constants;
mod endpoints;
mod error;
mod lib;

// Program entry point
#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = init_db().await;

    // Endpoints strictly for admins
    let admin_auth = Router::new()
        .route(
            formatcp!("{FACULTIES_ENDPOINT}/:id/{KEYWORD_MODIFY_ENDPOINT}"),
            get(endpoints::faculties::update_faculty_fe),
        )
        .route(
            formatcp!("{FACULTIES_ENDPOINT}/:id/{KEYWORD_MODIFY_ENDPOINT}"),
            post(endpoints::faculties::update_faculty),
        )
        .route(
            formatcp!("/faculties/:id/delete"),
            post(endpoints::faculties::delete_faculty),
        )
        .route(
            formatcp!("{FACULTIES_ENDPOINT}/{KEYWORD_CREATE_ENDPOINT}"),
            post(endpoints::faculties::create_faculty),
        )
        .route(
            formatcp!("{FACULTIES_ENDPOINT}/{KEYWORD_CREATE_ENDPOINT}"),
            get(endpoints::faculties::create_faculty_fe),
        )
        .route(
            formatcp!("{CLASSES_ENDPOINT}/{KEYWORD_CREATE_ENDPOINT}"),
            post(endpoints::classes::create_class),
        )
        .route(
            formatcp!("{CLASSES_ENDPOINT}/:id/{KEYWORD_REMOVE_ENDPOINT}"),
            post(endpoints::classes::delete_class),
        )
        .route(
            formatcp!("{CLASSES_ENDPOINT}/:id/{KEYWORD_MODIFY_ENDPOINT}"),
            post(endpoints::classes::update_class),
        )
        .route(
            formatcp!("{CLASSES_ENDPOINT}/:id/{KEYWORD_MODIFY_ENDPOINT}"),
            get(endpoints::classes::update_class_fe),
        )
        .route(
            formatcp!("{CLASSES_ENDPOINT}/{KEYWORD_CREATE_ENDPOINT}"),
            get(endpoints::classes::create_class_fe),
        )
        .route(
            "/open_close_dates",
            get(endpoints::open_close_date::get_page),
        )
        .route(
            "/open_close_dates",
            post(endpoints::open_close_date::update),
        )
        .route("/files/:id/delete", post(endpoints::classes_files::delete))
        .route(
            EXPORT_CSV_ENDPOINT,
            get(endpoints::administration::export_csv),
        )
        .route(
            EXPORT_JSON_ENDPOINT,
            get(endpoints::administration::export_json),
        )
        .route(
            MOVE_CHOICES_ENDPOINT,
            get(endpoints::administration::move_choices),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::auth_middleware,
        ));

    // Uploader
    let uploader = Router::new()
        .route(
            formatcp!("{CLASSES_ENDPOINT}/:id/{KEYWORD_UPLOAD_ENDPOINT}"),
            post(endpoints::classes_files::upload),
        )
        .layer(DefaultBodyLimit::max(MAX_CLASS_FILE_SIZE))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::auth_middleware,
        ));

    // For endpoints that have differences when the user is authed or the user isn't authed
    let auth_differences = Router::new()
        .route("/", get(endpoints::index::index))
        .route(
            formatcp!("{CLASSES_ENDPOINT}/:id"),
            get(endpoints::classes::view_class_fe),
        )
        .route(
            FACULTIES_ENDPOINT,
            get(endpoints::faculties::view_faculties_fe),
        )
        .route(CLASSES_ENDPOINT, get(endpoints::classes::filter_fe))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::permissive_middleware,
        ));

    // For endpoints that don't care if the user is authed or not
    let no_auth = Router::new()
        .route("/admin/login", get(endpoints::auth::admin_login_fe))
        .route("/admin/login", post(endpoints::auth::admin_login_handler))
        .route("/login", get(endpoints::auth::student_login_fe))
        .route("/login", post(endpoints::auth::student_login_handler));

    // Endpoints available only to students
    let student_auth = Router::new()
        .route(STUDENT_PICK_ENDPOINT, get(endpoints::choices::pick_fe))
        .route(STUDENT_PICK_ENDPOINT, post(endpoints::choices::pick))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            endpoints::auth::student_middleware,
        ));

    let app = Router::new()
        .nest("/", admin_auth)
        .nest("/", auth_differences)
        .nest("/", student_auth)
        .nest("/", no_auth)
        .nest("/", uploader)
        .with_state(state)
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(CompressionLayer::new());

    let listener = TcpListener::bind(&env::var("BIND_ADDRESS").unwrap())
        .await
        .expect("Cannot start server");

    fs::create_dir_all(ASSETS_CLASSES_LOCAL_PATH).unwrap();

    println!("DCT running.");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
