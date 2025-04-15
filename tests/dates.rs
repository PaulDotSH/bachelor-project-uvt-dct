mod db;

#[tokio::test]
async fn test_choices_open_date() {
    dotenvy::dotenv().unwrap();
    let app_state = db::init_db().await;
    
    // Clean up existing entries
    let _ = sqlx::query!("DELETE FROM choices_open_date")
        .execute(&app_state.postgres)
        .await;
    
    // Set future dates for opening and closing
    let start_date = time::OffsetDateTime::now_utc() + time::Duration::days(1);
    let end_date = start_date + time::Duration::days(7);
    
    // Insert test dates
    let _ = sqlx::query!(
        "INSERT INTO choices_open_date (id, start_date, end_date) VALUES (0, $1, $2)",
        start_date,
        end_date
    )
    .execute(&app_state.postgres)
    .await
    .unwrap();
    
    // Verify dates were inserted correctly
    let dates = sqlx::query!(
        "SELECT start_date, end_date FROM choices_open_date WHERE id = 0"
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();
    
    // Allow for slight time differences in database conversion
    let db_start = dates.start_date;
    let db_end = dates.end_date;
    
    let start_diff = (db_start.unix_timestamp() - start_date.unix_timestamp()).abs();
    let end_diff = (db_end.unix_timestamp() - end_date.unix_timestamp()).abs();
    
    // Time should be within 1 second difference due to potential rounding in DB
    assert!(start_diff <= 1, "Start date mismatch");
    assert!(end_diff <= 1, "End date mismatch");
    
    // Test date manipulation - update to current time
    let new_start_date = time::OffsetDateTime::now_utc();
    let new_end_date = new_start_date + time::Duration::days(14);
    
    let _ = sqlx::query!(
        "UPDATE choices_open_date SET start_date = $1, end_date = $2 WHERE id = 0",
        new_start_date,
        new_end_date
    )
    .execute(&app_state.postgres)
    .await
    .unwrap();
    
    // Verify update
    let updated_dates = sqlx::query!(
        "SELECT start_date, end_date FROM choices_open_date WHERE id = 0"
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();
    
    let updated_start_diff = (updated_dates.start_date.unix_timestamp() - new_start_date.unix_timestamp()).abs();
    let updated_end_diff = (updated_dates.end_date.unix_timestamp() - new_end_date.unix_timestamp()).abs();
    
    assert!(updated_start_diff <= 1, "Updated start date mismatch");
    assert!(updated_end_diff <= 1, "Updated end date mismatch");
    
    // Clean up
    let _ = sqlx::query!("DELETE FROM choices_open_date")
        .execute(&app_state.postgres)
        .await;
} 