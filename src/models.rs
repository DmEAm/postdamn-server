use std::fmt;
use std::fmt::{Display, Formatter};
use std::time::{SystemTime};
use diesel_ltree::Ltree;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub created_at: SystemTime,
}

#[derive(Queryable)]
pub struct Permission {
    pub id: Ltree,
    pub name: String,
    pub created_at: SystemTime,
}

#[derive(Queryable, Debug)]
pub struct User {
    pub id: Uuid,
    pub created_at: SystemTime,
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
impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "User (Id={}, Name={})", self.id, self.name)
    }
}
