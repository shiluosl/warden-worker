use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendType {
    Text = 0,
}

impl SendType {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Text),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRequest {
    #[serde(rename = "type")]
    pub r#type: i32,
    pub key: String,
    pub password: Option<String>,
    pub max_access_count: Option<Value>,
    pub expiration_date: Option<String>,
    pub deletion_date: String,
    #[serde(default)]
    pub disabled: bool,
    pub hide_email: Option<bool>,
    pub name: String,
    pub notes: Option<String>,
    pub text: Option<Value>,
    #[allow(dead_code)]
    pub file: Option<Value>,
    #[allow(dead_code)]
    pub file_length: Option<Value>,
    #[allow(dead_code)]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendAccessRequest {
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SendDBModel {
    pub id: String,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
    pub name: String,
    pub notes: Option<String>,
    #[serde(rename = "type")]
    pub r#type: i32,
    pub data: String,
    pub akey: String,
    pub password_hash: Option<String>,
    pub password_salt: Option<String>,
    pub password_iter: Option<i32>,
    pub max_access_count: Option<i32>,
    pub access_count: i32,
    pub created_at: String,
    pub updated_at: String,
    pub expiration_date: Option<String>,
    pub deletion_date: String,
    pub disabled: i32,
    pub hide_email: Option<i32>,
}

impl SendDBModel {
    pub fn access_id(&self) -> Option<String> {
        let uuid = Uuid::parse_str(&self.id).ok()?;
        Some(URL_SAFE_NO_PAD.encode(uuid.as_bytes()))
    }

    pub fn text_value(&self) -> Value {
        serde_json::from_str(&self.data).unwrap_or(Value::Null)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "accessId": self.access_id(),
            "type": self.r#type,
            "name": self.name,
            "notes": self.notes,
            "text": if self.r#type == SendType::Text as i32 { self.text_value() } else { Value::Null },
            "file": Value::Null,
            "key": self.akey,
            "maxAccessCount": self.max_access_count,
            "accessCount": self.access_count,
            "password": self.password_hash,
            "disabled": self.disabled != 0,
            "hideEmail": self.hide_email.map(|value| value != 0),
            "revisionDate": self.updated_at,
            "expirationDate": self.expiration_date,
            "deletionDate": self.deletion_date,
            "object": "send",
        })
    }

    pub fn to_access_json(&self, creator_identifier: Option<String>) -> Value {
        json!({
            "id": self.id,
            "type": self.r#type,
            "name": self.name,
            "text": if self.r#type == SendType::Text as i32 { self.text_value() } else { Value::Null },
            "file": Value::Null,
            "expirationDate": self.expiration_date,
            "creatorIdentifier": creator_identifier,
            "object": "send-access",
        })
    }
}
