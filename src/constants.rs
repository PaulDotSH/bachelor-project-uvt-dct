use chrono::{Duration, TimeDelta};
use crate::constant_parse::*;
// You can change this depending on your preferences
pub const INVALID_USER_PW: &'static str = "Incorrect username or password";
pub const TOKEN_EXPIRE_TIME: TimeDelta = Duration::days(7);
pub const BAD_DOT_ENV: &'static str = "Missing variable in .env file";
// Edit these variables ONLY from the .env file
pub const TOKEN_LENGTH: usize = parse_usize(env!("TOKEN_LENGTH"));
// Static files
pub static LOGIN_HTML: &str = include_str!("./static/login.html");
