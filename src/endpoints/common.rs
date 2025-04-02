#![allow(dead_code)]

use crate::error::AppError;
use async_trait::async_trait;
use axum::extract::{rejection::FormRejection, Form, FromRequest, Request};
use axum::http::HeaderMap;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use redis::aio::MultiplexedConnection;
use redis_pool::connection::RedisPoolConnection;
use redis_pool::SingleRedisPool;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use validator::Validate;
#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, Clone)]
pub struct Faculty {
    pub id: i32,
    pub name: String,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, Clone)]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub descr: String,
    pub faculty: i32,
    pub semester: Semester,
    pub requirements: Option<String>,
    pub prof: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[sqlx(type_name = "Semester", rename_all = "PascalCase")]
pub enum Semester {
    First,
    Second,
}

impl Display for Semester {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Semester::First => {
                write!(f, "First")
            }
            Semester::Second => {
                write!(f, "Second")
            }
        }
    }
}

impl TryFrom<&str> for Semester {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "First" => Ok(Semester::First),
            "Second" => Ok(Semester::Second),
            &_ => Err("Cannot parse Semester"),
        }
    }
}

impl FromStr for Semester {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "First" => Ok(Semester::First),
            "Second" => Ok(Semester::Second),
            &_ => Err("Cannot parse Semester"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AuthUserType {
    Guest,
    Student,
    Admin,
}

#[inline(always)]
pub fn generate_token(length: usize) -> String {
    Alphanumeric.sample_string(&mut thread_rng(), length)
}

#[inline(always)]
pub fn get_username_from_header_unchecked(headers: &HeaderMap) -> &str {
    headers
        .get("id")
        .expect("Header \"id\" doesn't exist")
        .to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_nr_mat_from_header_unchecked(headers: &HeaderMap) -> &str {
    headers
        .get("nr_mat")
        .expect("Header \"nr_mat\" doesn't exist")
        .to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_email_from_header_unchecked(headers: &HeaderMap) -> &str {
    headers
        .get("email")
        .expect("Header \"email\" doesn't exist")
        .to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_faculty_from_header_unchecked(headers: &HeaderMap) -> &str {
    headers
        .get("faculty")
        .expect("Header \"faculty\" doesn't exist")
        .to_str()
        .expect("Header cannot be converted into a string")
}

#[inline(always)]
pub fn get_username_from_header(headers: &HeaderMap) -> Option<&str> {
    headers.get("id").and_then(|id| id.to_str().ok())
}

#[inline(always)]
pub fn is_student_from_headers(headers: &HeaderMap) -> bool {
    headers.get("nr_mat").is_some()
}

#[inline(always)]
pub fn is_admin_from_headers(headers: &HeaderMap) -> bool {
    headers.get("id").is_some()
}

#[inline(always)]
pub fn get_auth_type_from_headers(headers: &HeaderMap) -> AuthUserType {
    if is_admin_from_headers(headers) {
        return AuthUserType::Admin;
    }
    if is_student_from_headers(headers) {
        return AuthUserType::Student;
    }
    AuthUserType::Guest
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
        Some(s) => FromStr::from_str(s)
            .map_err(serde::de::Error::custom)
            .map(Some),
    }
}

// Trims the string based on newline and character count
pub fn trim_string(input: &str, max_newlines: u32, max_characters: usize) -> &str {
    let mut newline_count = 0;
    let mut char_count = 0;

    let index = input
        .char_indices()
        .find_map(|(i, c)| {
            if c == '\n' {
                newline_count += 1;
            }
            char_count += c.len_utf8();

            if newline_count > max_newlines || char_count >= max_characters {
                Some(i)
            } else {
                None
            }
        })
        .unwrap_or(input.len());

    &input[..index]
}

#[inline(always)]
pub async fn flush_redis_db_conn(conn: &mut RedisPoolConnection<MultiplexedConnection>) {
    redis::cmd("flushdb")
        .query_async::<_, Vec<u8>>(conn)
        .await
        .unwrap();
}

#[inline(always)]
pub async fn flush_redis_db(redis: &SingleRedisPool) {
    let mut conn = redis.aquire().await.unwrap();
    flush_redis_db_conn(&mut conn).await;
}
