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
    pub id: i64,
    pub cache_root: i64,
    pub cube_key: String,
    pub requests: i64,
    pub created: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name = "cuboids"]
pub struct NewCuboid {
    pub cache_root: i32,
    pub cube_key: String,
    pub requests: i64,
}
