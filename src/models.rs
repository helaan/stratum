use crate::schema::*;
use chrono::{DateTime, Utc};

#[derive(AsChangeset, Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "teams"]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub rights: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
