use argon2::{Argon2, PasswordHash, PasswordVerifier};

mod db;

#[tokio::test]
async fn test_account_creation() {
    dotenvy::dotenv().unwrap();
    let app_state = db::init_db().await;

    let result = sqlx::query!("SELECT * FROM users where username = 'Admin'")
        .fetch_one(&app_state.postgres)
        .await.unwrap();

    let parsed_hash_db = PasswordHash::new(&result.pass).unwrap();

    assert!(Argon2::default()
    .verify_password(std::env::var("DEFAULT_ADMIN_PASSWORD").expect("ENV VAR NOT SET").as_bytes(), &parsed_hash_db).is_ok())
}
