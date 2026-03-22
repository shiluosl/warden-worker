use axum::{
    extract::{Path, State},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;
use worker::{query, D1Database, Env};

use crate::{
    auth::Claims,
    crypto::{generate_salt, hash_password_for_storage, verify_password},
    db,
    error::AppError,
    handlers::ciphers::RawJson,
    models::send::{SendAccessRequest, SendDBModel, SendRequest, SendType},
};

const SEND_INACCESSIBLE_MSG: &str = "Send does not exist or is no longer available";
const SEND_PASSWORD_ITERATIONS: u32 = 100_000;

fn now_string() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn parse_rfc3339_utc(value: &str, field_name: &str) -> Result<String, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| {
            dt.with_timezone(&Utc)
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string()
        })
        .map_err(|_| AppError::BadRequest(format!("Invalid {}", field_name)))
}

fn ensure_text_send_type(value: i32) -> Result<(), AppError> {
    match SendType::from_i32(value) {
        Some(SendType::Text) => Ok(()),
        None => Err(AppError::BadRequest(
            "Only text sends are currently supported".to_string(),
        )),
    }
}

fn normalize_text_payload(text: Option<serde_json::Value>) -> Result<String, AppError> {
    let mut text =
        text.ok_or_else(|| AppError::BadRequest("Send data not provided".to_string()))?;
    let object = text
        .as_object_mut()
        .ok_or_else(|| AppError::BadRequest("Text send payload must be an object".to_string()))?;
    object.remove("response");
    serde_json::to_string(&text).map_err(|_| AppError::Internal)
}

fn parse_optional_i32(
    value: Option<serde_json::Value>,
    field_name: &str,
) -> Result<Option<i32>, AppError> {
    let Some(value) = value else {
        return Ok(None);
    };

    if value.is_null() {
        return Ok(None);
    }

    if let Some(number) = value.as_i64() {
        return i32::try_from(number)
            .map(Some)
            .map_err(|_| AppError::BadRequest(format!("Invalid {}", field_name)));
    }

    if let Some(text) = value.as_str() {
        return text
            .parse::<i32>()
            .map(Some)
            .map_err(|_| AppError::BadRequest(format!("Invalid {}", field_name)));
    }

    Err(AppError::BadRequest(format!("Invalid {}", field_name)))
}

fn validate_deletion_date(deletion_date: &str) -> Result<(), AppError> {
    let parsed = DateTime::parse_from_rfc3339(deletion_date)
        .map_err(|_| AppError::BadRequest("Invalid deletionDate".to_string()))?
        .with_timezone(&Utc);

    if parsed > Utc::now() + Duration::days(31) {
        return Err(AppError::BadRequest(
            "You cannot have a Send with a deletion date that far into the future. Adjust the Deletion Date to a value less than 31 days from now and try again.".to_string(),
        ));
    }

    Ok(())
}

async fn hash_send_password(password: &str) -> Result<(String, String, i32), AppError> {
    let salt = generate_salt()?;
    let hash = hash_password_for_storage(password, &salt, SEND_PASSWORD_ITERATIONS).await?;
    Ok((hash, salt, SEND_PASSWORD_ITERATIONS as i32))
}

async fn fetch_send_for_user(
    db: &D1Database,
    send_id: &str,
    user_id: &str,
) -> Result<SendDBModel, AppError> {
    query!(
        db,
        "SELECT * FROM sends WHERE id = ?1 AND user_id = ?2",
        send_id,
        user_id
    )
    .map_err(|_| AppError::Database)?
    .first(None)
    .await?
    .ok_or_else(|| AppError::NotFound("Send not found".to_string()))
}

async fn creator_identifier(
    db: &D1Database,
    send: &SendDBModel,
) -> Result<Option<String>, AppError> {
    if matches!(send.hide_email, Some(value) if value != 0) {
        return Ok(None);
    }

    let Some(user_id) = send.user_id.as_ref() else {
        return Ok(None);
    };

    db.prepare("SELECT email FROM users WHERE id = ?1")
        .bind(&[user_id.clone().into()])?
        .first::<String>(Some("email"))
        .await
        .map_err(|_| AppError::Database)
}

fn access_id_to_send_id(access_id: &str) -> Option<String> {
    let decoded = URL_SAFE_NO_PAD.decode(access_id.as_bytes()).ok()?;
    let uuid = Uuid::from_slice(&decoded).ok()?;
    Some(uuid.to_string())
}

fn is_send_unavailable(send: &SendDBModel) -> bool {
    if send.disabled != 0 {
        return true;
    }

    if let Some(max_access_count) = send.max_access_count {
        if send.access_count >= max_access_count {
            return true;
        }
    }

    let now = Utc::now();

    if let Some(expiration_date) = &send.expiration_date {
        if let Ok(expiration) = DateTime::parse_from_rfc3339(expiration_date) {
            if now >= expiration.with_timezone(&Utc) {
                return true;
            }
        }
    }

    if let Ok(deletion_date) = DateTime::parse_from_rfc3339(&send.deletion_date) {
        if now >= deletion_date.with_timezone(&Utc) {
            return true;
        }
    }

    false
}

pub(crate) async fn list_user_sends(
    db: &D1Database,
    user_id: &str,
) -> Result<Vec<SendDBModel>, AppError> {
    query!(
        db,
        "SELECT * FROM sends WHERE user_id = ?1 ORDER BY updated_at DESC",
        user_id
    )
    .map_err(|_| AppError::Database)?
    .all()
    .await?
    .results()
    .map_err(|_| AppError::Internal)
}

pub(crate) async fn purge_expired_sends(env: &Env) -> Result<u32, worker::Error> {
    let db: D1Database = env.d1("vault1")?;
    let now = now_string();

    #[derive(serde::Deserialize)]
    struct CountResult {
        count: u32,
    }

    #[derive(serde::Deserialize)]
    struct AffectedUser {
        user_id: Option<String>,
    }

    let affected_users: Vec<AffectedUser> = query!(
        &db,
        "SELECT DISTINCT user_id FROM sends WHERE deletion_date <= ?1 AND user_id IS NOT NULL",
        now
    )?
    .all()
    .await?
    .results()
    .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let count_result = query!(
        &db,
        "SELECT COUNT(*) as count FROM sends WHERE deletion_date <= ?1",
        now
    )?
    .first::<CountResult>(None)
    .await?;

    let count = count_result.map(|result| result.count).unwrap_or(0);
    if count == 0 {
        return Ok(0);
    }

    query!(&db, "DELETE FROM sends WHERE deletion_date <= ?1", now)?
        .run()
        .await?;

    for user in affected_users.into_iter().filter_map(|user| user.user_id) {
        query!(
            &db,
            "UPDATE users SET updated_at = ?1 WHERE id = ?2",
            now,
            user
        )?
        .run()
        .await?;
    }

    Ok(count)
}

#[worker::send]
pub async fn get_sends(claims: Claims, State(env): State<Arc<Env>>) -> Result<RawJson, AppError> {
    let db = db::get_db(&env)?;
    let sends = list_user_sends(&db, &claims.sub).await?;
    let sends_json: Vec<serde_json::Value> = sends.into_iter().map(|send| send.to_json()).collect();
    let response = serde_json::json!({
        "data": sends_json,
        "object": "list",
        "continuationToken": serde_json::Value::Null,
    });

    Ok(RawJson(
        serde_json::to_string(&response).map_err(|_| AppError::Internal)?,
    ))
}

#[worker::send]
pub async fn get_send(
    claims: Claims,
    State(env): State<Arc<Env>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = db::get_db(&env)?;
    let send = fetch_send_for_user(&db, &id, &claims.sub).await?;
    Ok(Json(send.to_json()))
}

#[worker::send]
pub async fn create_send(
    claims: Claims,
    State(env): State<Arc<Env>>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let SendRequest {
        r#type,
        key,
        password,
        max_access_count,
        expiration_date,
        deletion_date,
        disabled,
        hide_email,
        name,
        notes,
        text,
        file: _,
        file_length: _,
        id: _,
    } = payload;

    ensure_text_send_type(r#type)?;
    validate_deletion_date(&deletion_date)?;

    let db = db::get_db(&env)?;
    let now = now_string();
    let send_id = Uuid::new_v4().to_string();
    let data = normalize_text_payload(text)?;
    let expiration_date = expiration_date
        .as_deref()
        .map(|value| parse_rfc3339_utc(value, "expirationDate"))
        .transpose()?;
    let deletion_date = parse_rfc3339_utc(&deletion_date, "deletionDate")?;
    let max_access_count = parse_optional_i32(max_access_count, "maxAccessCount")?;
    let (password_hash, password_salt, password_iter) = match password.as_deref() {
        Some(password) => {
            let (hash, salt, iter) = hash_send_password(password).await?;
            (Some(hash), Some(salt), Some(iter))
        }
        None => (None, None, None),
    };
    let disabled = if disabled { 1 } else { 0 };
    let hide_email = hide_email.map(|value| if value { 1 } else { 0 });

    query!(
        &db,
        "INSERT INTO sends (id, user_id, organization_id, name, notes, type, data, akey, password_hash, password_salt, password_iter, max_access_count, access_count, created_at, updated_at, expiration_date, deletion_date, disabled, hide_email)
         VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, ?13, ?14, ?15, ?16, ?17)",
        send_id,
        claims.sub,
        name,
        notes,
        r#type,
        data,
        key,
        password_hash,
        password_salt,
        password_iter,
        max_access_count,
        now,
        now,
        expiration_date,
        deletion_date,
        disabled,
        hide_email,
    )
    .map_err(|_| AppError::Database)?
    .run()
    .await?;

    db::touch_user_updated_at(&db, &claims.sub).await?;
    let send = fetch_send_for_user(&db, &send_id, &claims.sub).await?;
    Ok(Json(send.to_json()))
}

#[worker::send]
pub async fn update_send(
    claims: Claims,
    State(env): State<Arc<Env>>,
    Path(id): Path<String>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let SendRequest {
        r#type,
        key,
        password,
        max_access_count,
        expiration_date,
        deletion_date,
        disabled,
        hide_email,
        name,
        notes,
        text,
        file: _,
        file_length: _,
        id: _,
    } = payload;

    ensure_text_send_type(r#type)?;
    validate_deletion_date(&deletion_date)?;

    let db = db::get_db(&env)?;
    let existing = fetch_send_for_user(&db, &id, &claims.sub).await?;
    if existing.r#type != r#type {
        return Err(AppError::BadRequest("Sends can't change type".to_string()));
    }

    let now = now_string();
    let data = normalize_text_payload(text)?;
    let expiration_date = expiration_date
        .as_deref()
        .map(|value| parse_rfc3339_utc(value, "expirationDate"))
        .transpose()?;
    let deletion_date = parse_rfc3339_utc(&deletion_date, "deletionDate")?;
    let max_access_count = parse_optional_i32(max_access_count, "maxAccessCount")?;
    let (password_hash, password_salt, password_iter) = match password {
        Some(password) => {
            let (hash, salt, iter) = hash_send_password(&password).await?;
            (Some(hash), Some(salt), Some(iter))
        }
        None => (
            existing.password_hash.clone(),
            existing.password_salt.clone(),
            existing.password_iter,
        ),
    };
    let disabled = if disabled { 1 } else { 0 };
    let hide_email = hide_email.map(|value| if value { 1 } else { 0 });

    query!(
        &db,
        "UPDATE sends SET name = ?1, notes = ?2, data = ?3, akey = ?4, password_hash = ?5, password_salt = ?6, password_iter = ?7, max_access_count = ?8, updated_at = ?9, expiration_date = ?10, deletion_date = ?11, disabled = ?12, hide_email = ?13 WHERE id = ?14 AND user_id = ?15",
        name,
        notes,
        data,
        key,
        password_hash,
        password_salt,
        password_iter,
        max_access_count,
        now,
        expiration_date,
        deletion_date,
        disabled,
        hide_email,
        id,
        claims.sub,
    )
    .map_err(|_| AppError::Database)?
    .run()
    .await?;

    db::touch_user_updated_at(&db, &claims.sub).await?;
    let send = fetch_send_for_user(&db, &id, &claims.sub).await?;
    Ok(Json(send.to_json()))
}

#[worker::send]
pub async fn delete_send(
    claims: Claims,
    State(env): State<Arc<Env>>,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let db = db::get_db(&env)?;
    fetch_send_for_user(&db, &id, &claims.sub).await?;

    query!(
        &db,
        "DELETE FROM sends WHERE id = ?1 AND user_id = ?2",
        id,
        claims.sub
    )
    .map_err(|_| AppError::Database)?
    .run()
    .await?;

    db::touch_user_updated_at(&db, &claims.sub).await?;
    Ok(Json(()))
}

#[worker::send]
pub async fn remove_password(
    claims: Claims,
    State(env): State<Arc<Env>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = db::get_db(&env)?;
    fetch_send_for_user(&db, &id, &claims.sub).await?;
    let now = now_string();

    query!(
        &db,
        "UPDATE sends SET password_hash = NULL, password_salt = NULL, password_iter = NULL, updated_at = ?1 WHERE id = ?2 AND user_id = ?3",
        now,
        id,
        claims.sub,
    )
    .map_err(|_| AppError::Database)?
    .run()
    .await?;

    db::touch_user_updated_at(&db, &claims.sub).await?;
    let send = fetch_send_for_user(&db, &id, &claims.sub).await?;
    Ok(Json(send.to_json()))
}

#[worker::send]
pub async fn access_send(
    State(env): State<Arc<Env>>,
    Path(access_id): Path<String>,
    Json(payload): Json<SendAccessRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let Some(send_id) = access_id_to_send_id(&access_id) else {
        return Err(AppError::NotFound(SEND_INACCESSIBLE_MSG.to_string()));
    };

    let db = db::get_db(&env)?;
    let mut send: SendDBModel = query!(&db, "SELECT * FROM sends WHERE id = ?1", send_id)
        .map_err(|_| AppError::Database)?
        .first(None)
        .await?
        .ok_or_else(|| AppError::NotFound(SEND_INACCESSIBLE_MSG.to_string()))?;

    if is_send_unavailable(&send) {
        return Err(AppError::NotFound(SEND_INACCESSIBLE_MSG.to_string()));
    }

    if let (Some(password_hash), Some(password_salt), Some(password_iter)) = (
        send.password_hash.as_deref(),
        send.password_salt.as_deref(),
        send.password_iter,
    ) {
        let Some(password) = payload.password.as_deref() else {
            return Err(AppError::Unauthorized("Password not provided".to_string()));
        };

        let valid =
            verify_password(password, password_hash, password_salt, password_iter as u32).await?;
        if !valid {
            return Err(AppError::Unauthorized("Invalid password".to_string()));
        }
    }

    send.access_count += 1;
    let now = now_string();
    query!(
        &db,
        "UPDATE sends SET access_count = ?1, updated_at = ?2 WHERE id = ?3",
        send.access_count,
        now,
        send.id,
    )
    .map_err(|_| AppError::Database)?
    .run()
    .await?;

    if let Some(user_id) = send.user_id.as_ref() {
        db::touch_user_updated_at(&db, user_id).await?;
    }

    send.updated_at = now;
    let creator_identifier = creator_identifier(&db, &send).await?;
    Ok(Json(send.to_access_json(creator_identifier)))
}
