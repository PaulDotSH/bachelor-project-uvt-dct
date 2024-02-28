use std::intrinsics::write_bytes;
use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, Redirect};
use sailfish::TemplateOnce;
use serde::Deserialize;
use sqlx::{query, query_as, query_scalar};
use tokio::fs::File;
use validator::Validate;
use crate::AppState;
use crate::endpoints::common::*;
use crate::error::AppError;

#[derive(Deserialize, Validate)]
pub struct NewClass {
    #[validate(length(min = 3, message = "Nume materie prea scurt"))]
    name: String,
    #[validate(length(min = 25, message = "Descriere prea scurta"))]
    descr: String,
    faculty: i32,
    semester: Semester,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    requirements: Option<String>,
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
    class: Class,
    files: Vec<ClassFile>,
    is_admin: bool
}

pub async fn view_class_fe(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>) -> Result<Html<String>, AppError> {
    let record = query!(
        r#"
        SELECT id, name, descr, faculty, semester::text, requirements, prof FROM classes WHERE id = $1 AND disabled = false;
        "#,
        id
    )
        .fetch_one(&state.postgres)
        .await?;

    let files = query_as!(
        ClassFile,
        r#"
        SELECT id, name FROM classes_files WHERE classes_id = $1;
        "#,
        id
    )
        .fetch_all(&state.postgres)
        .await?;

    let is_admin = is_admin_from_headers(&headers);

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
        },
        is_admin,
        files
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}

use tokio::io::AsyncWriteExt;
pub async fn upload(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>, mut multipart: Multipart)
    -> Result<Redirect, AppError> {
    println!("id in upload {}", id);
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let file_path = format!("./assets/classes/{}", file_name);
        let mut file = File::create(&file_path).await?;

        let data = field.bytes().await.unwrap();
        file.write_all(&data).await?;

        //TODO: insert into db
        if query!(
        r#"
        INSERT INTO classes_files(name, classes_id) VALUES ($1, $2);
        "#,
        file_name, id
        )
            .execute(&state.postgres)
            .await.is_err() {
                tokio::fs::remove_file(&file_path).await?;
            };

    }
    Ok(Redirect::to(format!("/classes/{}/edit", id).as_str()))
}

pub async fn delete_class(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Redirect, AppError> {
    query!(
        r#"
        DELETE FROM classes WHERE id = $1;
        "#,
        id,
    )
        .execute(&state.postgres)
        .await?;

    Ok(Redirect::to("/classes"))
}

// This isn't "duplicate" code, I chose this approach to have the flexibility of having different fields compared to
// when making a new class, this is a "tradeoff"
#[derive(Deserialize, Validate)]
pub struct UpdatedClass {
    #[validate(length(min = 3, message = "Nume materie prea scurt"))]
    name: String,
    #[validate(length(min = 25, message = "Descriere prea scurta"))]
    descr: String,
    faculty: i32,
    semester: Semester,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    requirements: Option<String>,
    #[validate(length(min = 8, message = "Nume profesor prea scurt"))]
    prof: String,
}

pub async fn update_class(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedForm(payload): ValidatedForm<UpdatedClass>,
) -> Result<Redirect, AppError> {
    query!(
        r#"
        UPDATE classes SET name = $1, descr = $2, faculty = $3, semester = $4, requirements = $5, prof = $6 WHERE id = $7;
        "#,
        payload.name,
        payload.descr,
        payload.faculty,
        payload.semester as Semester,
        payload.requirements,
        payload.prof,
        id
    )
        .execute(&state.postgres)
        .await?;

    Ok(Redirect::to("/classes"))
}

#[derive(TemplateOnce)]
#[template(path = "class_edit.stpl")]
struct EditClassTemplate {
    class: Class,
    faculties: Vec<Faculty>,
    files: Vec<ClassFile>
}

pub struct ClassFile {
    name: String,
    id: i32
}

pub async fn update_class_fe(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, AppError> {

    let class_record = query!(
        r#"
        SELECT id, name, descr, faculty, semester::text, requirements, prof FROM classes WHERE id = $1;
        "#,
        id
    )
        .fetch_one(&state.postgres)
        .await?;

    let files = query_as!(
        ClassFile,
        r#"
        SELECT id, name FROM classes_files WHERE classes_id = $1;
        "#,
        id
    )
        .fetch_all(&state.postgres)
        .await?;

    let faculties = query_as!(
        Faculty,
        r#"
        SELECT * FROM faculties;
        "#
    )
        .fetch_all(&state.postgres)
        .await?;

    //TODO: Fix this duplication
    let ctx = EditClassTemplate {
        class: Class {
            id,
            name: class_record.name,
            descr: class_record.descr,
            faculty: class_record.faculty, //TODO: Add faculty name...
            semester: match class_record.semester.unwrap().as_ref() {
                "First" => Semester::First,
                "Second" => Semester::Second,
                _ => panic!("Unexpected semester value"),
            },
            requirements: class_record.requirements,
            prof: class_record.prof,
        },
        faculties,
        files
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}