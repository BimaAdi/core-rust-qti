use redis::{Connection, ConnectionLike};
use serde::{Deserialize, Serialize};

use crate::{model::user::User, settings::Config};

// use super::security::Claims;

pub fn get_redis_connection(redis_url: &str) -> anyhow::Result<Connection> {
    let client = redis::Client::open(redis_url)?;
    let con = client.get_connection()?;
    Ok(con)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub refresh_token: String,
}

pub fn add_session<C: ConnectionLike>(
    redis_conn: &mut C,
    user: &User,
    config: &Config,
    token: String,
    refresh_token: String,
) -> anyhow::Result<()> {
    // let token_exp_date = *now + Duration::minutes(config.jwt_exp as i64);
    let session_data = SessionData {
        user_id: user.id.to_string(),
        refresh_token,
    };
    let session_json = serde_json::to_string(&session_data)?;
    redis::Cmd::set_ex(token, session_json, config.jwt_exp as u64).exec(redis_conn)?;
    Ok(())
}

pub fn get_session<C: ConnectionLike>(
    redis_conn: &mut C,
    token: String,
) -> anyhow::Result<Option<SessionData>> {
    let res: Option<String> = redis::cmd("get").arg(token).query(redis_conn)?;
    if res.is_none() {
        return Ok(None);
    }
    let res = res.unwrap();
    let session_data: SessionData = serde_json::from_str(res.as_str())?;
    Ok(Some(session_data))
}

pub fn remove_session<C: ConnectionLike>(
    redis_conn: &mut C,
    token: String,
) -> anyhow::Result<bool> {
    let res: Option<String> = redis::cmd("get").arg(&token).query(redis_conn)?;
    if res.is_none() {
        return Ok(false);
    }
    let res = res.unwrap();
    let session_data: SessionData = serde_json::from_str(res.as_str())?;
    redis::cmd("del")
        .arg(session_data.refresh_token)
        .exec(redis_conn)?;
    redis::cmd("del").arg(token).exec(redis_conn)?;
    Ok(true)
}
