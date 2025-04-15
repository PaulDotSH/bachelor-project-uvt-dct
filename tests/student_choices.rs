mod db;

#[tokio::test]
async fn test_student_class_choices_integration() {
    dotenvy::dotenv().unwrap();
    let app_state = db::init_db().await;
    
    // Create test data - faculty, student, and classes
    let faculty_result = sqlx::query!(
        "INSERT INTO faculties (name) VALUES ('Test Integration Faculty') ON CONFLICT DO NOTHING RETURNING id"
    )
    .fetch_optional(&app_state.postgres)
    .await
    .unwrap();
    
    let test_faculty = match faculty_result {
        Some(record) => record.id,
        None => {
            sqlx::query!("SELECT id FROM faculties WHERE name = 'Test Integration Faculty'")
                .fetch_one(&app_state.postgres)
                .await
                .unwrap()
                .id
        }
    };
    
    // Create unique student for this test
    let test_student_id = "INTEGRATION_TEST_STU";
    let test_student_email = "integration_test@example.com";
    
    // Clean up any existing test data to ensure clean state
    let _ = sqlx::query!("DELETE FROM choices WHERE nr_mat = $1", test_student_id)
        .execute(&app_state.postgres)
        .await;
    let _ = sqlx::query!("DELETE FROM old_choices WHERE nr_mat = $1", test_student_id)
        .execute(&app_state.postgres)
        .await;
    let _ = sqlx::query!("DELETE FROM students WHERE nr_mat = $1", test_student_id)
        .execute(&app_state.postgres)
        .await;
    
    // Insert test student
    let _ = sqlx::query!(
        "INSERT INTO students (nr_mat, email, cnp3, faculty, token) 
         VALUES ($1, $2, '123', $3, 'integration_test_token')",
        test_student_id,
        test_student_email,
        test_faculty
    )
    .execute(&app_state.postgres)
    .await
    .unwrap();
    
    // Create unique test classes for this integration test
    let _ = sqlx::query!("DELETE FROM classes WHERE name LIKE 'Integration Test Class %'")
        .execute(&app_state.postgres)
        .await;
    
    let class1 = sqlx::query!(
        "INSERT INTO classes (name, descr, faculty, semester, prof)
         VALUES ('Integration Test Class 1', 'Desc 1', $1, 'First'::Semester, 'Prof 1')
         RETURNING id",
        test_faculty
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap()
    .id;
    
    let class2 = sqlx::query!(
        "INSERT INTO classes (name, descr, faculty, semester, prof)
         VALUES ('Integration Test Class 2', 'Desc 2', $1, 'First'::Semester, 'Prof 2')
         RETURNING id",
        test_faculty
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap()
    .id;
    
    // Test Case 1: Student selects their first and second choice
    let _ = sqlx::query!(
        "INSERT INTO choices (nr_mat, first_choice, second_choice)
         VALUES ($1, $2, $3)",
        test_student_id,
        class1,
        class2
    )
    .execute(&app_state.postgres)
    .await
    .unwrap();
    
    // Verify the choices were saved correctly
    let student_choices = sqlx::query!(
        "SELECT nr_mat, first_choice, second_choice FROM choices WHERE nr_mat = $1",
        test_student_id
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();
    
    assert_eq!(student_choices.nr_mat, test_student_id);
    assert_eq!(student_choices.first_choice, class1);
    assert_eq!(student_choices.second_choice, class2);
    
    // Test Case 2: Student can't select the same class twice
    // This should fail because of the CHECK constraint
    let duplicate_result = sqlx::query!(
        "INSERT INTO choices (nr_mat, first_choice, second_choice)
         VALUES ($1, $2, $2) 
         ON CONFLICT (nr_mat) DO UPDATE SET first_choice = $2, second_choice = $2",
        test_student_id,
        class1
    )
    .execute(&app_state.postgres)
    .await;
    
    assert!(duplicate_result.is_err());
    
    // Test Case 3: Past classes get recorded in old_choices
    let _ = sqlx::query!(
        "INSERT INTO old_choices (nr_mat, choice) VALUES ($1, $2)",
        test_student_id,
        class1
    )
    .execute(&app_state.postgres)
    .await
    .unwrap();
    
    let old_choice = sqlx::query!(
        "SELECT choice FROM old_choices WHERE nr_mat = $1 AND choice = $2",
        test_student_id,
        class1
    )
    .fetch_optional(&app_state.postgres)
    .await
    .unwrap();
    
    assert!(old_choice.is_some());
    assert_eq!(old_choice.unwrap().choice, class1);
    
    // Clean up all test data
    let _ = sqlx::query!("DELETE FROM choices WHERE nr_mat = $1", test_student_id)
        .execute(&app_state.postgres)
        .await;
    let _ = sqlx::query!("DELETE FROM old_choices WHERE nr_mat = $1", test_student_id)
        .execute(&app_state.postgres)
        .await;
    let _ = sqlx::query!("DELETE FROM students WHERE nr_mat = $1", test_student_id)
        .execute(&app_state.postgres)
        .await;
    let _ = sqlx::query!("DELETE FROM classes WHERE id = $1 OR id = $2", class1, class2)
        .execute(&app_state.postgres)
        .await;
} 