use diesel::prelude::*;
use diesel::result::Error;
use diesel::dsl::{max};

use crate::models;

pub fn find_map_by_keystr(
    req: &str,
    conn: &SqliteConnection
) -> Result<Option<models::Map>, Error>
{
    use super::schema::maps::dsl::*;
    
    let map = maps
        .filter(keystr.eq(req))
        .first::<models::Map>(conn)
        .optional()?;
    Ok(map)
}

pub fn count_points(
    conn: &SqliteConnection
) -> Result<i32, Error> 
{
    use super::schema::points::dsl::*;
    if let Some(max_id) = points.select(max(id)).first::<Option<i32>>(conn)? {
        Ok(max_id + 1)
    } else {
        Ok(1)
    }
}

pub fn insert_point(point: &models::Point, conn: &SqliteConnection) -> Result<(), Error>
{
    use super::schema::points::dsl::*;
    diesel::insert_into(points).values(point).execute(conn)?;
    Ok(())
}

pub fn get_points_in_map(map_key: &str, conn: &SqliteConnection) -> Result<Vec<models::Point>, Error>
{
    use super::schema::*;

    points::table.inner_join(maps::table)
        .filter(maps::keystr.eq(map_key))
        .select(models::POINT_COLUMNS)
        .load::<models::Point>(conn)
}

pub fn delete_points_in_map(map_key: &str, id: i32, conn: &SqliteConnection) -> Result<(), Error>
{
    use super::schema::*;
    diesel::delete(points::table
        .filter(points::id.eq(id))
        .filter(points::mapid.eq_any( maps::table.filter(maps::keystr.eq(&map_key)).select(maps::id) ))
    ).execute(conn)?;
    Ok(())
}

pub fn modify_point_in_map(map_key: &str, req: &models::PointUpdate, conn: &SqliteConnection) -> Result<(), Error>
{
    println!("Modify {:?}", req);
    use super::schema::*;
    diesel::update(points::table
        .filter(points::id.eq(req.id))
        .filter(points::mapid.eq_any( maps::table.filter(maps::keystr.eq(&map_key)).select(maps::id) ))
    )
    .set(req)
    .execute(conn)?;
    Ok(())
}