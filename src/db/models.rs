/*

Copyright 2020 The Johns Hopkins University Applied Physics Laboratory

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

*/

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
