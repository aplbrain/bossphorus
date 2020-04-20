use super::schema;
use chrono::prelude::*;
use diesel::*;
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

#[derive(Debug, Identifiable, Insertable, PartialEq, Queryable)]
pub struct Cuboid {
    pub id: i64,
    pub cache_root: i32,
    pub cube_key: String,
    pub requests: i64,
    pub created: NaiveDateTime,
    pub last_accessed: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "cuboids"]
pub struct NewCuboid {
    pub cache_root: i32,
    pub cube_key: String,
    pub requests: i64,
}
