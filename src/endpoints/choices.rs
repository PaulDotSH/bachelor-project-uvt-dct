use axum::extract::State;
use axum::Form;
use axum::http::HeaderMap;
use axum::response::{Html, Redirect};
use chrono::NaiveDateTime;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use validator::Validate;

use crate::AppState;
use crate::collect_with_capacity::*;
use crate::endpoints::common::*;
use crate::error::AppError;

struct StudentChoice {
    first_choice: i32,
    second_choice: i32,
    created: NaiveDateTime,
    updated: Option<NaiveDateTime>,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
struct TinyClass {
    pub id: i32,
    pub name: String,
    pub semester: Semester,
}

#[derive(TemplateOnce)]
#[template(path = "choice_pick.stpl")]
struct PickChoiceTemplate<'a> {
    fs_classes: &'a [TinyClass],
    ss_classes: &'a [TinyClass],
    choices: Option<StudentChoice>,
}

pub async fn pick_fe(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    let nr_mat = get_nr_mat_from_header_unchecked(&headers);
    let student_faculty = 1;

    let records = query!(
        r#"
        SELECT id, name, semester::text FROM classes WHERE faculty != $1 ORDER BY semester;
        "#,
        student_faculty,
    )
    .fetch_all(&state.postgres)
    .await?;

    let choices = query_as!(
        StudentChoice,
        r#"
        SELECT created, updated, first_choice, second_choice FROM choices WHERE nr_mat = $1
        "#,
        nr_mat
    )
    .fetch_optional(&state.postgres)
    .await?;

    let mut split_idx = 0;

    let len = records.len();
    let classes = records
        .into_iter()
        .enumerate()
        .map(|(i, record)| {
            let semester = match record.semester.unwrap().as_ref() {
                "First" => Semester::First,
                "Second" => {
                    if split_idx == 0 {
                        split_idx = i;
                    }
                    Semester::Second
                }
                _ => panic!("Unexpected semester value"),
            };

            TinyClass {
                id: record.id,
                name: record.name,
                semester,
            }
        })
        .collect_with_capacity(len);

    let ctx = PickChoiceTemplate {
        fs_classes: &classes[0..split_idx],
        ss_classes: &classes[split_idx..],
        choices,
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;

    Ok(Html::from(body))
}

#[derive(Deserialize, Validate, Debug)]
pub struct Choice {
    first: i32,
    second: i32,
}

pub async fn pick(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<Choice>,
) -> Result<Redirect, AppError> {
    //TODO: Check if the user selected a class from it's faculty

    let nr_mat = get_nr_mat_from_header_unchecked(&headers);

    query!(
        r#"
        INSERT INTO choices (nr_mat, first_choice, second_choice)
        VALUES ($1, $2, $3)
        ON CONFLICT (nr_mat) DO UPDATE
        SET first_choice = EXCLUDED.first_choice,
            second_choice = EXCLUDED.second_choice,
            updated = NOW();
        "#,
        nr_mat,
        payload.first,
        payload.second
    )
    .execute(&state.postgres)
    .await?;

    Ok(Redirect::to("/pick"))
}
