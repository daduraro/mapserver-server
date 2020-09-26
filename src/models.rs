// use serde::{Deserialize, Serialize};

// use crate::db::schema::maps;

use super::db::schema::points;

#[derive(Debug, Clone, Queryable)]
pub struct Map {
    pub id: i32,
    pub keystr: String,
    pub fpath: String,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[table_name = "points"]
pub struct Point {
    pub id: i32,
    pub mapid: i32,
    pub coordx: f32,
    pub coordy: f32,
    pub title: Option<String>,
    pub body: Option<String>,
}

// ugh...
type PointColumns = (
    points::id,
    points::mapid,
    points::coordx,
    points::coordy,
    points::title,
    points::body,
);
pub const POINT_COLUMNS: PointColumns = (
    points::id,
    points::mapid,
    points::coordx,
    points::coordy,
    points::title,
    points::body,
);

#[derive(AsChangeset, Debug)]
#[table_name="points"]
pub struct PointUpdate {
    pub id: i32,
    pub title: Option<Option<String>>,
    pub body: Option<Option<String>>,
}
