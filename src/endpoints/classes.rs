use axum::extract::{Path, State};
use axum::response::{Html, Redirect};
use sailfish::TemplateOnce;
use serde::Deserialize;
use sqlx::{query, query_as, query_scalar};
use validator::Validate;
use crate::AppState;
use crate::endpoints::common::{Class, Faculty, Semester, ValidatedForm};
use crate::error::AppError;

#[derive(Deserialize, Validate)]
pub struct NewClass {
    #[validate(length(min = 3, message = "Nume materie prea scurt"))]
    name: String,
    #[validate(length(min = 25, message = "Descriere prea scurta"))]
    descr: String,
    faculty: i32,
    semester: Semester,
    requirements: String,
    #[validate(length(min = 8, message = "Nume profesor prea scurt"))]
    prof: String,
}

#[derive(TemplateOnce)]
#[template(path = "class_create.stpl")]
struct CreateClassTemplate {
    faculties: Vec<Faculty>
}

pub async fn create_class_fe(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let faculties = query_as!(
        Faculty,
        r#"
        SELECT * FROM faculties;
        "#
    )
        .fetch_all(&state.postgres)
        .await?;

    let ctx = CreateClassTemplate {
        faculties
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}

pub async fn create_class(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<NewClass>,
) -> Result<Redirect, AppError> {
    let id: i32 = query_scalar!(
        r#"
        INSERT INTO classes(name, descr, faculty, semester, requirements, prof) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id;
        "#,
        payload.name,
        payload.descr,
        payload.faculty,
        payload.semester as Semester, // needed for custom enums
        payload.requirements,
        payload.prof
    )
        .fetch_one(&state.postgres)
        .await?;

    Ok(Redirect::to(format!("/classes/{}", id).as_str()))
}

#[derive(TemplateOnce)]
#[template(path = "class_view.stpl")]
struct ViewClassTemplate {
    class: Class
}

pub async fn view_class_fe(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Html<String>, AppError> {
    let record = query!(
        r#"
        SELECT id, name, descr, faculty, semester::text, requirements, prof FROM classes WHERE id = $1 AND disabled = false;
        "#,
        id
    )
        .fetch_one(&state.postgres)
        .await?;


    let ctx = ViewClassTemplate {
        class: Class {
            id,
            name: record.name,
            descr: record.descr,
            faculty: record.faculty, //TODO: Add faculty name...
            semester: match record.semester.unwrap().as_ref() {
                "First" => Semester::First,
                "Second" => Semester::Second,
                _ => panic!("Unexpected semester value"),
            },
            requirements: record.requirements,
            prof: record.prof,
        }
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}