mod db;

#[tokio::test]
async fn test_choices_crud() {
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

    let test_student = "TEST123";
    let _ = sqlx::query!(
        "INSERT INTO students (nr_mat, email, cnp3, faculty, token) 
         VALUES ($1, 'test@example.com', '123', $2, 'test_token') 
         ON CONFLICT (nr_mat) DO NOTHING",
        test_student,
        test_faculty
    )
    .execute(&app_state.postgres)
    .await;

    let class1 = sqlx::query!(
        "INSERT INTO classes (name, descr, faculty, semester, prof)
         VALUES ('Test Class 1', 'Test Description 1', $1, 'First', 'Test Professor 1')
         ON CONFLICT (id) DO UPDATE SET name = 'Test Class 1'
         RETURNING id",
        test_faculty
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap()
    .id;

    let class2 = sqlx::query!(
        "INSERT INTO classes (name, descr, faculty, semester, prof)
         VALUES ('Test Class 2', 'Test Description 2', $1, 'First', 'Test Professor 2')
         ON CONFLICT (id) DO UPDATE SET name = 'Test Class 2'
         RETURNING id",
        test_faculty
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap()
    .id;

    // Clean up any existing choices for test student
    let _ = sqlx::query!("DELETE FROM choices WHERE nr_mat = $1", test_student)
        .execute(&app_state.postgres)
        .await;

    // Create a choice
    let _ = sqlx::query!(
        "INSERT INTO choices (nr_mat, first_choice, second_choice) VALUES ($1, $2, $3)",
        test_student,
        class1,
        class2
    )
    .execute(&app_state.postgres)
    .await;

    // Read the choice
    let choice = sqlx::query!(
        "SELECT nr_mat, first_choice, second_choice FROM choices WHERE nr_mat = $1",
        test_student
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();

    assert_eq!(choice.nr_mat, test_student);
    assert_eq!(choice.first_choice, class1);
    assert_eq!(choice.second_choice, class2);

    // Update the choice
    let _ = sqlx::query!(
        "UPDATE choices SET first_choice = $2, second_choice = $3, updated = NOW() WHERE nr_mat = $1",
        test_student,
        class2,
        class1
    )
    .execute(&app_state.postgres)
    .await;

    // Read the updated choice
    let updated_choice = sqlx::query!(
        "SELECT nr_mat, first_choice, second_choice FROM choices WHERE nr_mat = $1",
        test_student
    )
    .fetch_one(&app_state.postgres)
    .await
    .unwrap();

    assert_eq!(updated_choice.nr_mat, test_student);
    assert_eq!(updated_choice.first_choice, class2);
    assert_eq!(updated_choice.second_choice, class1);

    // Delete the choice
    let _ = sqlx::query!("DELETE FROM choices WHERE nr_mat = $1", test_student)
        .execute(&app_state.postgres)
        .await;

    // Verify deletion
    let deleted_choice = sqlx::query!(
        "SELECT nr_mat FROM choices WHERE nr_mat = $1",
        test_student
    )
    .fetch_optional(&app_state.postgres)
    .await
    .unwrap();

    assert!(deleted_choice.is_none());

    // Clean up
    let _ = sqlx::query!("DELETE FROM students WHERE nr_mat = $1", test_student)
        .execute(&app_state.postgres)
        .await;
} 