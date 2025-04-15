use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;

#[test]
fn test_password_hashing_and_verification() {
    let password = "test_password123";
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    let hash_string = password_hash.to_string();
    
    let parsed_hash = PasswordHash::new(&hash_string).unwrap();
    
    let verification_result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
    assert!(verification_result.is_ok());
    
    let wrong_password = "wrong_password";
    let wrong_verification = Argon2::default().verify_password(wrong_password.as_bytes(), &parsed_hash);
    assert!(wrong_verification.is_err());
}

#[test]
fn test_multiple_hash_generation() {
    let password = "same_password";
    
    let salt1 = SaltString::generate(&mut OsRng);
    let salt2 = SaltString::generate(&mut OsRng);
    
    let argon2 = Argon2::default();
    
    let hash1 = argon2.hash_password(password.as_bytes(), &salt1).unwrap().to_string();
    let hash2 = argon2.hash_password(password.as_bytes(), &salt2).unwrap().to_string();
    
    assert_ne!(hash1, hash2);
    
    let parsed_hash1 = PasswordHash::new(&hash1).unwrap();
    let parsed_hash2 = PasswordHash::new(&hash2).unwrap();
    
    assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash1).is_ok());
    assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash2).is_ok());
}

#[test]
fn test_hash_format() {
    let password = "format_test_password";
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
    
    assert!(hash.starts_with("$argon2"));
    
    let parsed_hash = PasswordHash::new(&hash);
    assert!(parsed_hash.is_ok());
} 