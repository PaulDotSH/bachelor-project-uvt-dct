mod db;

#[tokio::test]
async fn test_classes_crud() {
    dotenvy::dotenv().unwrap();
    let app_state = db::init_db().await;
    
    // Create or get the test faculty
    let faculty_result = sqlx::query!(
        "INSERT INTO faculties (name) VALUES ('Test Faculty') ON CONFLICT DO NOTHING RETURNING id"
    )
    .fetch_optional(&app_state.postgres)
    .await
    .unwrap();
    
    let test_faculty = match faculty_result {
        Some(record) => record.id,
        None => {
            sqlx::query!("SELECT id FROM faculties WHERE name = 'Test Faculty'")
                .fetch_one(&app_state.postgres)
                .await
                .unwrap()
                .id
        }
    };
    
    // Test class name (unique for this test)
    let test_class_name = "Test Class CRUD";
    
    // Clean up any existing test class with this name
    let _ = sqlx::query!(
        "DELETE FROM classes WHERE name = $1", 
        test_class_name
    )
    .execute(&app_state.postgres)
    .await;
    
    // Create - Insert a new class
    let inserted_class = sqlx::query!(
        "INSERT INTO classes (name, descr, faculty, semester, prof)
         VALUES ($1, 'Test Description', $2, 'First'::Semester, 'Test Professor')
         RETURNING id",
        test_class_name,
        test_faculty
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();
    
    let class_id = inserted_class.id;
    
    // Read - Verify the class was created
    let class = sqlx::query!(
        "SELECT id, name, descr, faculty, semester::text as semester_text, prof FROM classes WHERE id = $1",
        class_id
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();
    
    assert_eq!(class.name, test_class_name);
    assert_eq!(class.descr, "Test Description");
    assert_eq!(class.faculty, test_faculty);
    assert_eq!(class.semester_text.as_ref().unwrap(), "First");
    assert_eq!(class.prof, "Test Professor");
    
    // Update - Change class description and professor
    let new_description = "Updated Test Description";
    let new_professor = "Updated Test Professor";
    
    let _ = sqlx::query!(
        "UPDATE classes SET descr = $1, prof = $2 WHERE id = $3",
        new_description,
        new_professor,
        class_id
    )
    .execute(&app_state.postgres)
    .await;
    
    // Read - Verify the updates
    let updated_class = sqlx::query!(
        "SELECT id, name, descr, faculty, semester::text as semester_text, prof FROM classes WHERE id = $1",
        class_id
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();
    
    assert_eq!(updated_class.name, test_class_name);
    assert_eq!(updated_class.descr, new_description);
    assert_eq!(updated_class.semester_text.as_ref().unwrap(), "First");
    assert_eq!(updated_class.prof, new_professor);
    
    // Delete - Remove the test class
    let _ = sqlx::query!(
        "DELETE FROM classes WHERE id = $1", 
        class_id
    )
    .execute(&app_state.postgres)
    .await;
    
    // Verify deletion
    let deleted_class = sqlx::query!(
        "SELECT id FROM classes WHERE id = $1",
        class_id
    )
    .fetch_optional(&app_state.postgres)
    .await
    .unwrap();
    
    assert!(deleted_class.is_none());
} 