use chrono::{Duration, TimeDelta};
use crate::constant_parse::*;
// You can change this depending on your preferences
pub const INVALID_USER_PW: &'static str = "Incorrect username or password";
pub const TOKEN_EXPIRE_TIME: TimeDelta = Duration::days(7);
pub const BAD_DOT_ENV: &'static str = "Missing variable in .env file";
pub const TOKEN_LENGTH: usize = 128;
// Static files
pub static LOGIN_HTML: &str = include_str!("./static/login.html");
