use crate::schema::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Session {
    pub key: Uuid,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(AsChangeset, Identifiable, Queryable, Serialize, Deserialize, Debug)]
#[table_name = "teams"]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(AsChangeset, Identifiable, Queryable, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub team_id: Option<i64>,
    pub username: String,
    pub password_hash: String,
    pub rights: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
