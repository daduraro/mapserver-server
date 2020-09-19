// use serde::{Deserialize, Serialize};

// use crate::db::schema::maps;

#[derive(Debug, Clone, Queryable)]
pub struct Map {
    pub id: i32,
    pub keystr: String,
    pub fpath: String,
}