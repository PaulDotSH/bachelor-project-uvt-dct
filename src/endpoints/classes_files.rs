use anyhow::anyhow;
use axum::extract::{Multipart, Path, State};
use axum::response::Redirect;
use sqlx::{query, query_scalar};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::constants::*;

use crate::error::AppError;
use crate::AppState;

pub async fn delete(
    State(state): State<AppState>,
    Path(file_id): Path<i32>,
) -> Result<Redirect, AppError> {
    let file_name = query_scalar!(
        r#"
        SELECT name FROM classes_files WHERE id = $1;
        "#,
        file_id
    )
    .fetch_one(&state.postgres)
    .await?;

    let mut transaction = state.postgres.begin().await?;

    query!(
        r#"
        DELETE FROM classes_files WHERE id = $1;
        "#,
        file_id
    )
    .execute(&mut *transaction)
    .await?;

    let file_path = format!("{ASSETS_CLASSES_LOCAL_PATH}/{file_name}");
    tokio::fs::remove_file(&file_path).await?;
    transaction.commit().await?;

    Ok(Redirect::to("/classes"))
}

pub async fn upload(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Redirect, AppError> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let file_path = format!("{ASSETS_CLASSES_LOCAL_PATH}/{file_name}");

        let mut file = File::create(&file_path).await?;

        let data = field.bytes().await.unwrap();
        file.write_all(&data).await?;

        if query!(
            r#"
        INSERT INTO classes_files(name, classes_id) VALUES ($1, $2);
        "#,
            file_name,
            id
        )
        .execute(&state.postgres)
        .await
        .is_err()
        {
            tokio::fs::remove_file(&file_path).await?;
            return Err(AppError(anyhow!("Could not insert file into db")));
        };
    }
    Ok(Redirect::to(format!("/classes/{}/edit", id).as_str()))
}
