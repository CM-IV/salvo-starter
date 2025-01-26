use crate::{
    app_error::AppError,
    app_writer::AppResult,
    config::DB,
    dtos::user::{
        UserAddRequest, UserLoginRequest, UserLoginResponse, UserResponse, UserUpdateRequest,
    },
    entities::users::User,
    middleware::jwt::get_token,
    utils::rand_utils,
};
use uuid::Uuid;

pub async fn add_user(req: UserAddRequest) -> AppResult<UserResponse> {
    let db = DB.get().ok_or(AppError::DatabaseConnectionFailed)?;
    let id = Uuid::new_v4().to_string();
    let hash_password = rand_utils::hash_password(req.password)
        .await
        .map_err(|e| AppError::PasswordHashError(e.to_string()))?;
    let _ = sqlx::query!(
        r#"
            INSERT INTO users (id, username, password)
            VALUES ($1, $2, $3)
            "#,
        id,
        req.username,
        hash_password,
    )
    .execute(db)
    .await?;

    Ok(UserResponse {
        id,
        username: req.username,
    })
}

pub async fn login(req: UserLoginRequest) -> AppResult<UserLoginResponse> {
    let db = DB.get().ok_or(AppError::DatabaseConnectionFailed)?;
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT id, username, password FROM users
            WHERE username = $1
            "#,
        req.username
    )
    .fetch_optional(db)
    .await?;
    if user.is_none() {
        return Err(AppError::UserNotFound.into());
    }
    let user = user.unwrap();
    if rand_utils::verify_password(req.password, user.password)
        .await
        .is_err()
    {
        return Err(AppError::InvalidPassword.into());
    }
    let (token, exp) = get_token(user.username.clone(), user.id.clone())
        .map_err(|_| AppError::TokenGenerationFailed)?;

    let res = UserLoginResponse {
        id: user.id,
        username: user.username,
        token,
        exp,
    };
    Ok(res)
}

pub async fn update_user(req: UserUpdateRequest) -> AppResult<UserResponse> {
    let db = DB.get().ok_or(AppError::DatabaseConnectionFailed)?;
    let hash_password = rand_utils::hash_password(req.password)
        .await
        .map_err(|e| AppError::PasswordHashError(e.to_string()))?;
    let _ = sqlx::query!(
        r#"
            UPDATE users
            SET username = $1, password = $2
            WHERE id = $3
            "#,
        req.username,
        hash_password,
        req.id,
    )
    .execute(db)
    .await?;

    Ok(UserResponse {
        id: req.id,
        username: req.username,
    })
}

pub async fn delete_user(id: String) -> AppResult<()> {
    let db = DB.get().ok_or(AppError::DatabaseConnectionFailed)?;
    sqlx::query!(
        r#"
            DELETE FROM users
            WHERE id = $1
            "#,
        id,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn users() -> AppResult<Vec<UserResponse>> {
    let db = DB.get().ok_or(AppError::DatabaseConnectionFailed)?;
    let users = sqlx::query_as!(
        User,
        r#"
            SELECT id, username, password FROM users
            "#,
    )
    .fetch_all(db)
    .await?;
    let res = users
        .into_iter()
        .map(|user| UserResponse {
            id: user.id,
            username: user.username,
        })
        .collect::<Vec<_>>();
    Ok(res)
}
