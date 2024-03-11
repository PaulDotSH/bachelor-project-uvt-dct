use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use chrono::NaiveDateTime;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use crate::AppState;
use crate::endpoints::classes::Filter;
use crate::endpoints::common::{get_nr_mat_from_header_unchecked, Semester};
use crate::error::AppError;

//TODO: Change faculty id to faculty name in the first query, using join

struct StudentChoice {
    first_choice: i32,
    second_choice: i32,
    created: NaiveDateTime,
    updated: Option<NaiveDateTime>
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
    choices: Option<StudentChoice>
}

pub async fn pick_fe(State(state): State<AppState>, headers: HeaderMap) -> Result<Html<String>, AppError> {
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

        let mut split_idx= 0;
        let classes: Vec<TinyClass> = records.into_iter().enumerate().map(|(i, record)| {
            let semester = match record.semester.unwrap().as_ref() {
                "First" => Semester::First,
                "Second" => {
                    if split_idx == 0 {
                        split_idx = i;
                    }
                    Semester::Second
                },
                _ => panic!("Unexpected semester value"),
            };

        TinyClass {
            id: record.id,
            name: record.name,
            semester,
        }}).collect();

        let ctx = PickChoiceTemplate {
            fs_classes: &classes[0..split_idx],
            ss_classes: &classes[split_idx..],
            choices,
        };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;

    Ok(Html::from(body))
}