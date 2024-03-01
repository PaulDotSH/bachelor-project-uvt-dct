use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::response::Redirect;
use sqlx::query;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::AppState;
use crate::error::AppError;

pub async fn upload(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>, mut multipart: Multipart)
                    -> Result<Redirect, AppError> {
    println!("id in upload {}", id);
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let file_path = format!("./assets/classes/{}", file_name);
        let mut file = File::create(&file_path).await?;

        let data = field.bytes().await.unwrap();
        file.write_all(&data).await?;

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