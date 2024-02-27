use async_trait::async_trait;
use axum::http::HeaderMap;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use serde::de::DeserializeOwned;
use axum::{
    extract::{rejection::FormRejection, Form, FromRequest, Request},
};
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use validator::Validate;

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Faculty {
    pub id: i32,
    pub name: String
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub descr: String,
    pub faculty: i32,
    pub semester: Semester,
    pub requirements: Option<String>,
    pub prof: String
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "Semester", rename_all = "PascalCase")]
pub enum Semester {
    First,
    Second
}

pub fn generate_token(length: usize) -> String {
    Alphanumeric.sample_string(&mut thread_rng(), length)
}

#[inline(always)]
pub fn get_username_from_header_unchecked(headers: &HeaderMap) -> &str {
    let id = headers.get("id").expect("Header \"id\" doesn't exist");
    id.to_str()
        .expect("Header cannot be converted into a string")
}

pub fn get_username_from_header(headers: &HeaderMap) -> Option<&str> {
    let id = headers.get("id").expect("Header \"id\" doesn't exist");
    id.to_str().ok()
}

pub fn log_action_from_headers(headers: &HeaderMap, ) {
    log_action(get_username_from_header_unchecked(headers));
}

pub fn log_action(username: &str) {
    // TODO: Implement
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);
#[async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
    where
        T: DeserializeOwned + Validate,
        S: Send + Sync,
        Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

