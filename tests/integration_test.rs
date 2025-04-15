mod db;

#[tokio::test]
async fn test_database_interaction() {
    dotenvy::dotenv().unwrap();
    let app_state = db::init_db().await;

    let test_username = "TestUser";
    let test_password = "TestPassword";
    let test_token = "TestToken";

    let _ = sqlx::query!("DELETE FROM users WHERE username = $1", test_username)
        .execute(&app_state.postgres)
        .await;

    let _ = sqlx::query!(
        "INSERT INTO users (username, pass, token, tok_expire) 
         VALUES ($1, $2, $3, NOW() + INTERVAL '7 days')",
        test_username,
        test_password,
        test_token
    )
    .execute(&app_state.postgres)
    .await;

    let result = sqlx::query!("SELECT username FROM users WHERE username = $1", test_username)
        .fetch_one(&app_state.postgres)
        .await;

    println!("{:?}", result);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().username, test_username);

    let cleanup = sqlx::query!("DELETE FROM users WHERE username = $1", test_username)
        .execute(&app_state.postgres)
        .await;
    
    assert!(cleanup.is_ok());
    
    let verification = sqlx::query!("SELECT username FROM users WHERE username = $1", test_username)
        .fetch_optional(&app_state.postgres)
        .await
        .unwrap();
    
    assert!(verification.is_none(), "Test user wasn't properly cleaned up");
}
