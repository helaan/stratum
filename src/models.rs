use std::time::SystemTime;
use crate::schema::teams;

#[derive(Queryable)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub rights: i16,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
