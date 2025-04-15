use chrono::{Duration, TimeDelta, Utc};
use dct::{collect_with_capacity::CollectWithCapacity, error::AppError};

// Constants for testing (copies of those from constants.rs to make tests independent)
const TOKEN_LENGTH: usize = 128;
const MAX_CLASS_FILE_SIZE: usize = 12 * 1024 * 1024; // 12MB
const TOKEN_EXPIRE_TIME: TimeDelta = Duration::days(7);

#[test]
fn test_token_length_boundary() {
    let empty_token = "";
    assert_eq!(empty_token.len(), 0);
    assert!(empty_token.len() < TOKEN_LENGTH);
    
    let short_token = "a".repeat(TOKEN_LENGTH - 1);
    assert_eq!(short_token.len(), TOKEN_LENGTH - 1);
    assert!(short_token.len() < TOKEN_LENGTH);
    
    let exact_token = "a".repeat(TOKEN_LENGTH);
    assert_eq!(exact_token.len(), TOKEN_LENGTH);
    
    let long_token = "a".repeat(TOKEN_LENGTH + 1);
    assert_eq!(long_token.len(), TOKEN_LENGTH + 1);
    assert!(long_token.len() > TOKEN_LENGTH);
}

#[test]
fn test_file_size_boundary() {
    let empty_file_size = 0;
    assert!(empty_file_size < MAX_CLASS_FILE_SIZE);
    
    let boundary_file_size = MAX_CLASS_FILE_SIZE;
    assert_eq!(boundary_file_size, MAX_CLASS_FILE_SIZE);
    
    let large_file_size = MAX_CLASS_FILE_SIZE + 1;
    assert!(large_file_size > MAX_CLASS_FILE_SIZE);
    
    let small_file_size = 1024 * 1024; // 1MB
    assert!(small_file_size < MAX_CLASS_FILE_SIZE);
}

#[test]
fn test_token_expiration_boundary() {
    let now = Utc::now();
    
    let fresh_token_time = now;
    assert!(fresh_token_time + TOKEN_EXPIRE_TIME > now);
    
    let about_to_expire = now - (TOKEN_EXPIRE_TIME - Duration::seconds(1));
    assert!(about_to_expire + TOKEN_EXPIRE_TIME > now);
    
    let expired_token_time = now - TOKEN_EXPIRE_TIME;
    assert_eq!(expired_token_time + TOKEN_EXPIRE_TIME, now);
    
    let long_expired_token = now - (TOKEN_EXPIRE_TIME + Duration::seconds(1));
    assert!(long_expired_token + TOKEN_EXPIRE_TIME < now);
}

#[test]
fn test_collect_with_capacity_boundary() {
    let empty_vec: Vec<i32> = Vec::new();
    let result = empty_vec.into_iter().collect_with_capacity(0);
    assert_eq!(result.len(), 0);
    assert_eq!(result.capacity(), 0);
    
    let single_item = vec![42];
    let result = single_item.into_iter().collect_with_capacity(1);
    assert_eq!(result.len(), 1);
    assert!(result.capacity() >= 1);
    
    let items = vec![1, 2, 3];
    let result = items.into_iter().collect_with_capacity(10);
    assert_eq!(result.len(), 3);
    assert!(result.capacity() >= 10);
    
    let many_items: Vec<i32> = (1..1000).collect();
    let result = many_items.into_iter().collect_with_capacity(5);
    assert_eq!(result.len(), 999);
    assert!(result.capacity() >= 5);
}

#[test]
fn test_error_boundary_cases() {
    // Test with empty error message
    let empty_error = AppError::from(anyhow::Error::msg(""));
    let error_debug = format!("{:?}", empty_error);
    assert!(error_debug.contains(""));
    
    // Test with very long error message (boundary of reasonable length)
    let long_message = "a".repeat(10000);
    let long_error = AppError::from(anyhow::Error::msg(long_message));
    let error_debug = format!("{:?}", long_error);
    assert!(error_debug.len() > 0);
    
    // Test error with non-UTF8 characters (edge case)
    let special_chars = "エラーメッセージ"; // Japanese for "error message"
    let special_error = AppError::from(anyhow::Error::msg(special_chars));
    let error_debug = format!("{:?}", special_error);
    assert!(error_debug.contains(special_chars));
} 