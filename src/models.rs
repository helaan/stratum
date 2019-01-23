use crate::schema::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct ContestProblem {
    pub contest_id: i64,
    pub problem_id: i64,
    pub label: String,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Contest {
    pub id: i64,
    pub name: String,
    pub short_name: String,
    pub start_at: Option<DateTime<Utc>>,
    pub freeze_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug)]
pub struct Problem {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug)]
#[belongs_to(Problem)]
pub struct ProblemStatement {
    pub id: i64,
    pub problem_id: i64,
    pub filename: String,
    pub mimetype: String,
    pub statement: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
