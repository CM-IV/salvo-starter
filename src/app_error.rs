use salvo::http::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("http::ParseError:`{0}`")]
    ParseError(#[from] ParseError),
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("Database connection failed")]
    DatabaseConnectionFailed,
    #[error("ValidationError:`{0}`")]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("PasswordHashError:`{0}`")]
    PasswordHashError(String),
    #[error("InvalidPassword")]
    InvalidPassword,
    #[error("TaskPanicked")]
    TaskPanicked,
    #[error("User does not exist")]
    UserNotFound,
    #[error("Failed to generate token")]
    TokenGenerationFailed,
}
