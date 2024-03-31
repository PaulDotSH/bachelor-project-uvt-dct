use chrono::{Duration, TimeDelta};
use const_format::formatcp;

// You can change this depending on your preferences
// Error messages
pub const INVALID_ADMIN_USER_PW: &str = "Incorrect username or password";
pub const INVALID_STUDENT_DETAILS: &str = "Incorrect nr_mat, email or cnp";
pub const PICKED_CLASS_FROM_OWN_FACULTY: &str =
    "You cannot pick a class from your own faculty";
pub const BAD_DOT_ENV: &str = "Missing variable in .env file";
pub const INVALID_TOKEN: &str = "Invalid token";
// Variables
pub const TOKEN_EXPIRE_TIME: TimeDelta = Duration::days(7);
pub const MAX_CLASS_FILE_SIZE: usize = 12 * 1024 * 1024; //12MB
pub const TOKEN_LENGTH: usize = 128;

// Static files
pub const ADMIN_LOGIN_HTML: &str = include_str!("./static/admin-login.html");
pub const STUDENT_LOGIN_HTML: &str = include_str!("./static/student-login.html");
// Paths
pub const ASSETS_CLASSES_PATH: &str = "/assets/classes";
pub const ASSETS_CLASSES_LOCAL_PATH: &str = formatcp!(".{ASSETS_CLASSES_PATH}");

// Endpoints
pub const EXPORT_CSV_ENDPOINT: &str = "/export-csv";
pub const EXPORT_JSON_ENDPOINT: &str = "/export-json";
pub const MOVE_CHOICES_ENDPOINT: &str = "/move-choices";
pub const STUDENT_PICK_ENDPOINT: &str = "/pick";
pub const FACULTIES_ENDPOINT: &str = "/faculties";
pub const CLASSES_ENDPOINT: &str = "/classes";
pub const KEYWORD_CREATE_ENDPOINT: &str = "new";
pub const KEYWORD_MODIFY_ENDPOINT: &str = "edit";
pub const KEYWORD_REMOVE_ENDPOINT: &str = "delete";
pub const KEYWORD_UPLOAD_ENDPOINT: &str = "upload";
