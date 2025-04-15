pub mod db;
pub mod unit;

// Import tests into this file
#[path = "accounts.rs"]
mod accounts_tests;

#[path = "choices.rs"]
mod choices_tests;

#[path = "classes.rs"]
mod classes_tests;

#[path = "dates.rs"]
mod dates_tests;

#[path = "integration_test.rs"]
mod integration_tests;

#[path = "student_choices.rs"]
mod student_choices_tests; 