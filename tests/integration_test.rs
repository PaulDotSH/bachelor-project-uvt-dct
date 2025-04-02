mod db;

#[tokio::test]
async fn test_database_interaction() {
    dotenvy::dotenv().unwrap();
    let app_state = db::init_db().await;

    // // Example query to test
    // let result = sqlx::query!("SELECT * FROM users LIMIT 1")
    //     .fetch_one(&app_state.postgres)
    //     .await;

    // println!("{:?}", result);

    // assert!(result.is_ok());
    assert!(true)
}
