// @generated automatically by Diesel CLI.

pub mod security {
    pub mod sql_types {
        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "ltree", schema = "security"))]
        pub struct Ltree;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Ltree;

        security.permissions (id) {
            id -> Ltree,
            created_at -> Timestamp,
            updated_at -> Nullable<Timestamp>,
            name -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Ltree;

        security.role_permissions (role_id, permission_id) {
            role_id -> Uuid,
            permission_id -> Ltree,
        }
    }

    diesel::table! {
        security.roles (id) {
            id -> Uuid,
            created_at -> Timestamp,
            updated_at -> Nullable<Timestamp>,
            name -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Ltree;

        security.user_permissions (user_id, permission_id) {
            user_id -> Uuid,
            permission_id -> Ltree,
        }
    }

    diesel::table! {
        security.user_roles (user_id, role_id) {
            user_id -> Uuid,
            role_id -> Uuid,
        }
    }

    diesel::table! {
        security.users (id) {
            id -> Uuid,
            created_at -> Timestamp,
            updated_at -> Nullable<Timestamp>,
            name -> Varchar,
            email -> Text,
            phone -> Text,
        }
    }

    diesel::joinable!(role_permissions -> permissions (permission_id));
    diesel::joinable!(role_permissions -> roles (role_id));
    diesel::joinable!(user_permissions -> permissions (permission_id));
    diesel::joinable!(user_permissions -> users (user_id));
    diesel::joinable!(user_roles -> roles (role_id));
    diesel::joinable!(user_roles -> users (user_id));

    diesel::allow_tables_to_appear_in_same_query!(
        permissions,
        role_permissions,
        roles,
        user_permissions,
        user_roles,
        users,
    );
}
