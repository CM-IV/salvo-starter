use anyhow::Result;
use jiff::{Timestamp, ToSpan};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Validation};
use salvo::jwt_auth::{ConstDecoder, CookieFinder, HeaderFinder, QueryFinder};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CFG;

const EPOCH: Timestamp = Timestamp::UNIX_EPOCH;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    username: String,
    user_id: String,
    exp: i64,
}

#[allow(dead_code)]
pub fn jwt_middleware() -> JwtAuth<JwtClaims, ConstDecoder> {
    let auth_handler: JwtAuth<JwtClaims, _> = JwtAuth::new(ConstDecoder::from_secret(
        CFG.jwt.jwt_secret.to_owned().as_bytes(),
    ))
    .finders(vec![
        Box::new(HeaderFinder::new()),
        Box::new(QueryFinder::new("token")),
        Box::new(CookieFinder::new("jwt_token")),
    ])
    .force_passed(false);
    auth_handler
}

#[allow(dead_code)]
pub fn get_token(username: String, user_id: String) -> Result<(String, i64)> {
    let exp = jiff::Timestamp::now()
        .checked_add(CFG.jwt.jwt_exp.second())
        .expect("Could not get jiff timestamp");
    let claim = JwtClaims {
        username,
        user_id,
        exp: EPOCH
            .until(exp)
            .expect("Could not get time since epoch")
            .get_seconds(),
    };
    let token: String = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &EncodingKey::from_secret(CFG.jwt.jwt_secret.as_bytes()),
    )?;
    Ok((
        token,
        EPOCH
            .until(exp)
            .expect("Could not get time since epoch")
            .get_seconds(),
    ))
}

#[allow(dead_code)]
pub fn decode_token(token: &str) -> bool {
    let validation = Validation::new(Algorithm::HS256);
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.jwt_secret.as_bytes()),
        &validation,
    )
    .is_ok()
}
