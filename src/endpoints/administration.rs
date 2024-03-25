use axum::extract::State;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Pool, Postgres};

use crate::error::AppError;
use crate::AppState;

async fn export_choices(pool: &Pool<Postgres>) -> Result<Vec<UserChoices>, sqlx::Error> {
    Ok(query_as!(
        UserChoices,
        r#"
        SELECT
    c.nr_mat,
    c.first_choice,
    fc.name AS first_choice_name,
    c.second_choice,
    sc.name AS second_choice_name,
    c.created,
    c.updated
FROM
    choices c
JOIN
    classes fc ON c.first_choice = fc.id
JOIN
    classes sc ON c.second_choice = sc.id;
        "#
    )
    .fetch_all(pool)
    .await?)
}

#[derive(Serialize, Deserialize, Debug)]
struct UserChoices {
    nr_mat: String,
    first_choice: i32,
    first_choice_name: String,
    second_choice: i32,
    second_choice_name: String,
    created: NaiveDateTime,
    updated: Option<NaiveDateTime>,
}

pub async fn export_csv(State(state): State<AppState>) -> Result<String, AppError> {
    let choices = export_choices(&state.postgres).await?;

    let mut wtr = csv::Writer::from_writer(vec![]);

    for uc in choices {
        wtr.serialize(uc)?;
    }

    Ok(String::from_utf8(wtr.into_inner()?)?)
}

pub async fn export_json(State(state): State<AppState>) -> Result<String, AppError> {
    let choices = export_choices(&state.postgres).await?;

    Ok(serde_json::to_string(&choices)?)
}

pub async fn move_choices(State(state): State<AppState>) -> Result<(), AppError> {
    let mut transaction = state.postgres.begin().await?;

    sqlx::query!(
        "INSERT INTO old_choices (nr_mat, choice)
         SELECT nr_mat, first_choice FROM choices"
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query!(
        "INSERT INTO old_choices (nr_mat, choice)
         SELECT nr_mat, second_choice FROM choices"
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query!("DELETE FROM choices")
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}
