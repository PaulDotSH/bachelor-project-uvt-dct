use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, Redirect};
use sailfish::TemplateOnce;
use serde::Deserialize;
use sqlx::{query, query_as};
use validator::Validate;

use crate::endpoints::common::*;
use crate::error::AppError;
use crate::AppState;

pub async fn create_faculty_fe() -> Result<Html<String>, AppError> {
    let ctx = CreateFacultyTemplate {};
    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}

#[derive(Deserialize, Validate)]
pub struct NewFaculty {
    #[validate(length(min = 3, message = "Nume facultate prea scurt"))]
    name: String,
}

pub async fn create_faculty(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<NewFaculty>,
) -> Result<Redirect, AppError> {
    query!(
        r#"
        INSERT INTO faculties(name) VALUES ($1);
        "#,
        payload.name,
    )
    .execute(&state.postgres)
    .await?;

    Ok(Redirect::to("/faculties"))
}

pub async fn delete_faculty(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Redirect, AppError> {
    query!(
        r#"
        DELETE FROM faculties WHERE id = $1;
        "#,
        id,
    )
    .execute(&state.postgres)
    .await?;

    Ok(Redirect::to("/faculties"))
}

#[derive(Deserialize, Validate)]
pub struct UpdatedFaculty {
    #[validate(length(min = 3, message = "Nume facultate prea scurt"))]
    name: String,
}

#[derive(TemplateOnce)]
#[template(path = "faculty_edit.stpl")]
struct EditFacultyTemplate {
    faculty: Faculty,
}

#[derive(TemplateOnce)]
#[template(path = "faculty_create.stpl")]
struct CreateFacultyTemplate {}

pub async fn update_faculty_fe(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, AppError> {
    let faculty = query_as!(
        Faculty,
        r#"
        SELECT * FROM faculties WHERE id = $1;
        "#,
        id
    )
    .fetch_one(&state.postgres)
    .await?;

    let ctx = EditFacultyTemplate { faculty };
    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}

pub async fn update_faculty(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedForm(payload): ValidatedForm<UpdatedFaculty>,
) -> Result<Redirect, AppError> {
    query!(
        r#"
        UPDATE faculties SET name = $1 WHERE id = $2;
        "#,
        payload.name,
        id
    )
    .execute(&state.postgres)
    .await?;

    Ok(Redirect::to("/faculties"))
}

#[derive(TemplateOnce)]
#[template(path = "faculties.stpl")]
struct ViewFacultiesTemplate {
    faculties: Vec<Faculty>,
    is_admin: bool,
}
pub async fn view_faculties_fe(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let faculties = query_as!(
        Faculty,
        r#"
        SELECT * FROM faculties;
        "#
    )
    .fetch_all(&state.postgres)
    .await?;

    let is_admin = is_admin_from_headers(&headers);

    let ctx = ViewFacultiesTemplate {
        faculties,
        is_admin,
    };
    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}
