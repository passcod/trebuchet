use super::schema::apps;
use super::schema::clients;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Queryable)]
pub struct Client {
    pub id: i32,
    pub connection: Uuid,
    pub connected: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub target: bool,
    pub app: String,
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(AsChangeset, Clone, Debug, Insertable)]
#[table_name = "clients"]
pub struct NewClient {
    pub connection: Uuid,
    pub target: bool,
    pub app: String,
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
pub struct App {
    pub id: i32,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub repo: String,
    pub build: Option<String>,
}

#[derive(AsChangeset, Clone, Debug, Insertable)]
#[table_name = "apps"]
pub struct NewApp {
    pub name: String,
    pub repo: String,
    pub build: Option<String>,
}
