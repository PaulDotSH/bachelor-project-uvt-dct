use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::query_scalar;
use time::{PrimitiveDateTime, OffsetDateTime, Duration};

use crate::{constants::*, lib::AppState};
use crate::endpoints::common::generate_token;
use crate::error::AppError;

// Serve the static files
pub async fn admin_login_fe() -> Html<&'static str> {
    Html::from(ADMIN_LOGIN_HTML)
}

pub async fn student_login_fe() -> Html<&'static str> {
    Html::from(STUDENT_LOGIN_HTML)
}

#[derive(sqlx::FromRow, Debug)]
struct User {
    username: String,
    tok_expire: PrimitiveDateTime,
}

#[derive(sqlx::FromRow, Debug)]
struct Student {
    nr_mat: String,
    email: String,
    tok_expire: PrimitiveDateTime,
    faculty: i32,
}

// Middleware that lets everyone in, however properly sets headers for admins and students
pub async fn permissive_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>, // insert the username and role headers in the following requests in case they are needed, so we don't hit the database again
    next: Next,                 // So we can forward the request
) -> Response {
    let headers = request.headers_mut();
    let _ = headers.remove("id"); // We do not care if the header did or did not exist
    let _ = headers.remove("nr_mat"); // We do not care if the header did or did not exist
                                      // We do not need to remove the other headers since our check if the request is sent by a student is done by checking the nr_mat
    let Some(cookie) = headers.get("cookie").cloned() else {
        return next.run(request).await;
    };

    let Ok(cookie_str) = cookie.to_str() else {
        return next.run(request).await;
    };

    let cookie_pairs: Vec<&str> = cookie_str.split(';').collect();

    for pair in cookie_pairs {
        if let Some(token) = pair.trim().strip_prefix("TOKEN=") {
            if token.is_empty() {
                return (StatusCode::INTERNAL_SERVER_ERROR, INVALID_TOKEN).into_response();
            }

            let Ok(user) = sqlx::query_as!(
                User,
                "SELECT username, tok_expire FROM users WHERE token = $1",
                token
            )
            .fetch_one(&state.postgres)
            .await
            else {
                return next.run(request).await;
            };

            if user.tok_expire < PrimitiveDateTime::new(
                OffsetDateTime::now_utc().date(),
                OffsetDateTime::now_utc().time()
            ) {
                return next.run(request).await;
            }

            headers.insert("id", HeaderValue::from_str(&user.username).unwrap());
        } else if let Some(token) = pair.trim().strip_prefix("STOKEN=") {
            if token.is_empty() {
                return (StatusCode::INTERNAL_SERVER_ERROR, INVALID_TOKEN).into_response();
            }

            let Ok(student) = sqlx::query_as!(
                Student,
                "SELECT nr_mat, email, tok_expire, faculty FROM students WHERE token = $1",
                token
            )
            .fetch_one(&state.postgres)
            .await
            else {
                return StatusCode::UNAUTHORIZED.into_response();
            };

            if student.tok_expire < PrimitiveDateTime::new(
                OffsetDateTime::now_utc().date(),
                OffsetDateTime::now_utc().time()
            ) {
                return next.run(request).await;
            }

            headers.insert("nr_mat", HeaderValue::from_str(&student.nr_mat).unwrap());
            headers.insert("email", HeaderValue::from_str(&student.email).unwrap());
            headers.insert(
                "faculty",
                HeaderValue::from_str(&student.faculty.to_string()).unwrap(),
            );
        }
    }

    next.run(request).await
}

// Middleware that only allows students
pub async fn student_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let headers = request.headers_mut();

    let Some(cookie) = headers.get("cookie").cloned() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(cookie_str) = cookie.to_str() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let cookie_pairs: Vec<&str> = cookie_str.split(';').collect();

    let mut ok = false;
    for pair in cookie_pairs {
        if let Some(token) = pair.trim().strip_prefix("STOKEN=") {
            if token.is_empty() {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid token").into_response();
            }

            let Ok(student) = sqlx::query_as!(
                Student,
                "SELECT nr_mat, email, tok_expire, faculty FROM students WHERE token = $1",
                token
            )
            .fetch_one(&state.postgres)
            .await
            else {
                return StatusCode::UNAUTHORIZED.into_response();
            };

            if student.tok_expire < PrimitiveDateTime::new(
                OffsetDateTime::now_utc().date(),
                OffsetDateTime::now_utc().time()
            ) {
                return next.run(request).await;
            }

            ok = true;
            headers.insert("nr_mat", HeaderValue::from_str(&student.nr_mat).unwrap());
            headers.insert("email", HeaderValue::from_str(&student.email).unwrap());
            headers.insert(
                "faculty",
                HeaderValue::from_str(&student.faculty.to_string()).unwrap(),
            );
        }
    }

    if !ok {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    next.run(request).await
}

// Middleware that only allows admins
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>, // insert the username headers in the following requests in case they are needed, so we don't hit the database again
    next: Next,                 // So we can forward the request
) -> Response {
    let headers = request.headers_mut();

    let Some(cookie) = headers.get("cookie").cloned() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(cookie_str) = cookie.to_str() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let cookie_pairs: Vec<&str> = cookie_str.split(';').collect();

    let mut ok = false;
    for pair in cookie_pairs {
        if let Some(token) = pair.trim().strip_prefix("TOKEN=") {
            if token.is_empty() {
                return (StatusCode::INTERNAL_SERVER_ERROR, INVALID_TOKEN).into_response();
            }
            let Ok(user) = sqlx::query_as!(
                User,
                "SELECT username, tok_expire FROM users WHERE token = $1",
                token
            )
            .fetch_one(&state.postgres)
            .await
            else {
                return (StatusCode::INTERNAL_SERVER_ERROR, INVALID_TOKEN).into_response();
            };

            if user.tok_expire < PrimitiveDateTime::new(
                OffsetDateTime::now_utc().date(),
                OffsetDateTime::now_utc().time()
            ) {
                return StatusCode::UNAUTHORIZED.into_response();
            }

            ok = true;
            headers.insert("id", HeaderValue::from_str(&user.username).unwrap());
        }
    }

    if !ok {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}

#[derive(Serialize, Deserialize)]
pub struct AdminLogin {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct StudentLogin {
    nr_mat: String,
    email: String,
    cnp3: String,
}

// Post request for login
pub async fn admin_login_handler(
    State(state): State<AppState>,
    Form(payload): Form<AdminLogin>,
) -> Result<Response, AppError> {
    let Ok(pw): Result<String, _> = query_scalar!(
        r#"
        SELECT pass from users where username = $1
        "#,
        payload.username,
    )
    .fetch_one(&state.postgres)
    .await
    else {
        return Err(AppError(anyhow!(INVALID_ADMIN_USER_PW)));
    };

    let Ok(parsed_hash) = PasswordHash::new(&pw) else {
        return Err(AppError(anyhow!("Db malformed for user")));
    };

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError(anyhow!(INVALID_ADMIN_USER_PW)));
    }

    let token = generate_token(TOKEN_LENGTH);

    sqlx::query!(
        "UPDATE users SET token = $1, tok_expire = $2 WHERE username = $3",
        token,
        PrimitiveDateTime::new(
            OffsetDateTime::now_utc().date(),
            OffsetDateTime::now_utc().time()
        ) + Duration::days(1),
        payload.username
    )
    .execute(&state.postgres)
    .await?;

    let cookie = format!("TOKEN={}; Path=/; Max-Age=604800", &token);

    let mut resp = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .body(Body::empty())
        .unwrap();

    let headers = resp.headers_mut();
    headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
    headers.insert("location", "/".parse().unwrap());

    Ok(resp)
}

// Post request for students login
pub async fn student_login_handler(
    State(state): State<AppState>,
    Form(payload): Form<StudentLogin>,
) -> Result<Response, AppError> {
    let exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM students WHERE nr_mat = $1 AND email = $2 AND cnp3 = $3)",
        payload.nr_mat,
        payload.email,
        payload.cnp3
    )
    .fetch_one(&state.postgres)
    .await?;

    if !exists.exists.unwrap() {
        return Err(AppError(anyhow!(INVALID_STUDENT_DETAILS)));
    }

    let token = generate_token(TOKEN_LENGTH);

    sqlx::query!(
        "UPDATE students SET token = $1, tok_expire = $2 WHERE nr_mat = $3",
        token,
        PrimitiveDateTime::new(
            OffsetDateTime::now_utc().date(),
            OffsetDateTime::now_utc().time()
        ) + Duration::days(1),
        payload.nr_mat
    )
    .execute(&state.postgres)
    .await?;

    let cookie = format!("STOKEN={}; Path=/; Max-Age=604800", &token);

    let mut redirect_resp = Redirect::to("/").into_response();
    redirect_resp
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.parse().unwrap());

    Ok(redirect_resp)
}
