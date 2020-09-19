use diesel::prelude::*;

use crate::models;

pub fn find_map_by_keystr(
    req: &str,
    conn: &SqliteConnection
) -> Result<Option<models::Map>, diesel::result::Error>
{
    use super::schema::maps::dsl::*;
    
    let map = maps
        .filter(keystr.eq(req))
        .first::<models::Map>(conn)
        .optional()?;
    Ok(map)
}