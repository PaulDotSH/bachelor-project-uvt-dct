use chrono::{Duration, TimeDelta};

// You can change this depending on your preferences
pub const INVALID_ADMIN_USER_PW: &'static str = "Incorrect username or password";
pub const INVALID_STUDENT_DETAILS: &'static str = "Incorrect nr_mat, email or cnp";

pub const TOKEN_EXPIRE_TIME: TimeDelta = Duration::days(7);
pub const BAD_DOT_ENV: &'static str = "Missing variable in .env file";
pub const TOKEN_LENGTH: usize = 128;
// Static files
pub static ADMIN_LOGIN_HTML: &str = include_str!("./static/admin-login.html");
pub static STUDENT_LOGIN_HTML: &str = include_str!("./static/student-login.html");
