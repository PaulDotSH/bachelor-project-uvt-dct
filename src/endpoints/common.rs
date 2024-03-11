use std::str::FromStr;
use async_trait::async_trait;
use axum::http::HeaderMap;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use serde::de::DeserializeOwned;
use axum::{
    extract::{rejection::FormRejection, Form, FromRequest, Request},
};
use serde::{Deserialize, Deserializer, Serialize};
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
    pub prof: String,
    //TODO: Add class files
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[sqlx(type_name = "Semester", rename_all = "PascalCase")]
pub enum Semester {
    First,
    Second
}

impl FromStr for Semester {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "First" => Ok(Semester::First),
            "Second" => Ok(Semester::Second),
            &_ => Err("Cannot parse Semester")
        }
    }
}

#[inline(always)]
pub fn generate_token(length: usize) -> String {
    Alphanumeric.sample_string(&mut thread_rng(), length)
}

#[inline(always)]
pub fn get_username_from_header_unchecked(headers: &HeaderMap) -> &str {
    let id = headers.get("id").expect("Header \"id\" doesn't exist");
    id.to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_nr_mat_from_header_unchecked(headers: &HeaderMap) -> &str {
    let id = headers.get("nr_mat").expect("Header \"nr_mat\" doesn't exist");
    id.to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_email_from_header_unchecked(headers: &HeaderMap) -> &str {
    let id = headers.get("email").expect("Header \"email\" doesn't exist");
    id.to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_username_from_header(headers: &HeaderMap) -> Option<&str> {
    let id = headers.get("id");
    if id.is_none() {
        return None;
    }
    id.unwrap().to_str().ok()
}

#[inline(always)]
pub fn is_admin_from_headers(headers: &HeaderMap) -> bool {
    let id = headers.get("id");
    id.is_some()
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

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: std::fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(serde::de::Error::custom).map(Some),
    }
}