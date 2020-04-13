use super::schema;
use chrono::prelude::*;
use schema::cache_roots;
use schema::cuboids;

#[derive(Identifiable, Queryable)]
pub struct CacheRoot {
    pub id: i32,
    pub path: String,
}

#[derive(Insertable)]
#[table_name = "cache_roots"]
pub struct NewCacheRoot {
    pub path: String,
}

#[derive(Identifiable, Queryable)]
pub struct Cuboid {
    pub id: u32,
    pub cache_root: u32,
    pub cube_key: String,
    // Diesel doesn't support u32 for add expressions, currently.
    pub requests: i32,
    pub created: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name = "cuboids"]
pub struct NewCuboid {
    pub cache_root: i32,
    pub cube_key: String,
    // Diesel doesn't support u32 for add expressions, currently.
    pub requests: i32,
}
