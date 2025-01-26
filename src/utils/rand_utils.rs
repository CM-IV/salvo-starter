use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use rand::Rng;
use std::iter;

use crate::app_error::AppError;

#[allow(dead_code)]
#[inline]
pub fn random_string(limit: usize) -> String {
    iter::repeat(())
        .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(limit)
        .collect()
}

pub async fn verify_password(password: String, password_hash: String) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| AppError::PasswordHashError(format!("invalid password hash: {}", e)))?;
        let result = hash.verify_password(&[&Argon2::default()], password);
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::InvalidPassword),
        }
    })
    .await
    .map_err(|_| AppError::TaskPanicked)?
}

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || -> Result<String, AppError> {
        let salt = SaltString::generate(rand::thread_rng());
        Ok(PasswordHash::generate(Argon2::default(), password, &salt)
            .map_err(|e| {
                AppError::PasswordHashError(format!("failed to generate password hash: {}", e))
            })?
            .to_string())
    })
    .await
    .map_err(|_| AppError::TaskPanicked)?
}
