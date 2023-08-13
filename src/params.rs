use diesel::query_builder::{AsQuery, Query};
use diesel::query_dsl::methods::OffsetDsl;
use diesel::Table;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, serde::Serialize, serde::Deserialize, IntoParams, ToSchema)]
pub struct Page {
    pub(crate) offset: Option<i64>,
    pub(crate) limit: Option<i64>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: Some(10),
        }
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize, IntoParams, ToSchema)]
pub struct Search {
    pub(crate) q: Option<String>,
}
