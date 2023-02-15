use std::fmt;
use chrono::{DateTime, Utc, NaiveDateTime};
use chrono::serde::ts_seconds_option;
use diesel_ltree::Ltree;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Queryable)]
pub struct Role {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
}

#[derive(Queryable)]
pub struct Permission {
    pub id: Ltree,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
}

#[derive(Queryable, Serialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Queryable)]
pub struct UserRole {
    pub role_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Queryable)]
pub struct RolePermission {
    pub permission_id: Ltree,
    pub role_id: Uuid,
}

#[derive(Queryable)]
pub struct UserPermission {
    pub permission_id: Ltree,
    pub user_id: Uuid,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User (Id={}, Name={})", self.id, self.name)
    }
}
