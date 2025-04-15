use axum::http::StatusCode;
use axum::response::IntoResponse;
use dct::error::AppError;
use futures_util::future::FutureExt;

#[test]
fn test_app_error_from_string() {
    let error_message = "Test error message";
    let app_error = AppError::from(anyhow::Error::msg(error_message));
    
    // The anyhow error should be wrapped correctly
    assert!(format!("{:?}", app_error).contains("Test error message"));
}

#[test]
fn test_app_error_into_response() {
    let error_message = "Test error message";
    let app_error = AppError::from(anyhow::Error::msg(error_message));
    
    let response = app_error.into_response();
    
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    
    // The body should contain the error message - we need to convert the response to bytes and then to a string to check
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .now_or_never()
        .unwrap()
        .unwrap();
    let body_string = String::from_utf8(bytes.to_vec()).unwrap();
    
    assert!(body_string.contains("Test error message"));
}

#[test]
fn test_app_error_from_different_error_types() {
    // Create error from std::io::Error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let app_error = AppError::from(io_error);
    assert!(format!("{:?}", app_error).contains("File not found"));
    
    // Create error from sqlx::Error (using a mock)
    let sqlx_error = sqlx::Error::RowNotFound;
    let app_error = AppError::from(sqlx_error);
    assert!(format!("{:?}", app_error).contains("no rows returned"));
}

#[test]
fn test_error_chain() {
    // Test error chaining through the '?' operator (simulated)
    let original_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let context_error = anyhow::Error::new(original_error).context("Failed to read file");
    let app_error = AppError::from(context_error);
    
    let error_string = format!("{:?}", app_error);
    assert!(error_string.contains("Permission denied"));
    assert!(error_string.contains("Failed to read file"));
} 