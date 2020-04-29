/// SQL database module.
pub mod models;
pub mod schema;

extern crate chrono;
extern crate diesel;
use super::config;
use super::usage_manager::UsageManager;
use chrono::prelude::*;
use diesel::prelude::*;
use models::{CacheRoot, Cuboid, NewCacheRoot, NewCuboid};
use std::collections::HashMap;
use std::fs;
use std::option::Option;
use std::path::Path;
use std::result::Result;

#[cfg(test)]
pub mod tests;

pub trait LeastRecentlyUsed {
    /// Find the `num` least recently used cuboids in the cache.
    ///
    /// # Arguments:
    ///
    /// * `num` - How many cuboids to retrieve.
    fn find_lru(&self, num: u32) -> Vec<Cuboid>;
}

pub struct SqliteCacheManager {
    /// The connection to the DB.
    connection: SqliteConnection,
    /// id of the cache root in the `cache_roots` table.  Need for inserts into
    /// `cuboids` table.
    cache_root_id: i32,
    /// Store all cache roots encountered during execution.
    cache_root_map: HashMap<i32, String>,
    /// The byte length of the CUBOID_ROOT_PATH.
    path_len: usize,
}

impl LeastRecentlyUsed for SqliteCacheManager {
    fn find_lru(&self, num: u32) -> Vec<Cuboid> {
        use schema::cuboids::dsl::*;
        cuboids
            .order(last_accessed)
            .limit(num as i64)
            .load::<Cuboid>(&self.connection)
            .expect("Error getting LRU cuboids")
    }
}

impl UsageManager for SqliteCacheManager {
    fn log_request(&self, key: String) {
        use schema::cuboids::dsl::*;

        // Strip off the root folder because the root, itself, is stored in
        // the `cache_roots` table.
        let (_root, remainder) = &key.split_at(self.path_len);

        match diesel::update(cuboids.filter(cube_key.eq(remainder)))
            .set((
                requests.eq(requests + 1),
                last_accessed.eq(Utc::now().naive_utc().to_string()),
            ))
            .execute(&self.connection)
        {
            Err(err) => println!("Error updating DB: {}", err),
            Ok(num_rows) => {
                if num_rows < 1 {
                    let new_request = NewCuboid {
                        cache_root: self.cache_root_id,
                        cube_key: remainder.to_string(),
                        requests: 1,
                    };
                    diesel::insert_into(cuboids)
                        .values(&new_request)
                        .execute(&self.connection)
                        .unwrap_or_else(|err| {
                            println!("insert failed: {}", err);
                            0
                        });
                }
            }
        }
    }
}

impl SqliteCacheManager {
    /// Constructor.
    ///
    /// # Arguments:
    ///
    /// * `db_url` - Connection string for the Sqlite DB
    pub fn new(db_url: &str) -> SqliteCacheManager {
        let connection =
            SqliteConnection::establish(db_url).expect(&format!("Error connecting to {}", db_url));
        SqliteCacheManager::init(connection)
    }

    /// Completes setup of the manager.  Called directly by the `new()` constructor.
    ///
    /// # Arguments:
    ///
    /// * `connection` - Open Sqlite connection
    fn init(connection: SqliteConnection) -> SqliteCacheManager {
        let cache_root_id = SqliteCacheManager::get_cache_root_id(&connection);
        let mut cache_root_map = HashMap::new();
        cache_root_map.insert(cache_root_id, config::CUBOID_ROOT_PATH.to_string());
        let path_len = config::CUBOID_ROOT_PATH.len();

        return SqliteCacheManager {
            connection,
            cache_root_id,
            cache_root_map,
            path_len,
        };
    }

    /// Remove the given list of cuboids from the cache.
    ///
    /// # Arguments
    ///
    /// * `unwanted` - List of cuboids to remove from the cache
    pub fn clean_cache(&mut self, unwanted: Vec<Cuboid>) {
        for cuboid in unwanted.iter() {
            let root_path = self.get_cache_root_path_from_map(cuboid.cache_root);
            if root_path.is_some() {
                let root_path = root_path.unwrap();
                if self
                    .remove_cuboid_file(&format!("{}{}", root_path, cuboid.cube_key))
                    .is_err()
                {
                    // ToDo: write to log.
                    println!("Error removing {}{}", root_path, cuboid.cube_key);
                    continue;
                }
                self.remove_cuboid_entry(cuboid.id)
            }
        }
    }

    /// Looks up the id of the cache root
    ///
    /// # Arguments
    ///
    /// * `connection` - Open connection to the DB
    fn get_cache_root_id(connection: &SqliteConnection) -> i32 {
        use schema::cache_roots::dsl::*;
        let row: Result<CacheRoot, diesel::result::Error> = cache_roots
            .filter(path.eq(config::get_cuboid_root_abs_path()))
            .get_result(connection);
        match row {
            Ok(row) => row.id,
            Err(_) => {
                let row = NewCacheRoot {
                    path: config::get_cuboid_root_abs_path(),
                };
                diesel::insert_into(cache_roots)
                    .values(row)
                    .execute(connection)
                    .expect("Could not update database");
                SqliteCacheManager::get_cache_root_id(connection)
            }
        }
    }

    /// Get the cache root path from the internal hashmap.  If it doesn't
    /// exist in the hashmap, load the path from the DB and add it to the
    /// hashmap.
    ///
    /// # Arguments
    ///
    /// * `root_id` - Cache root id in the DB
    fn get_cache_root_path_from_map(&mut self, root_id: i32) -> Option<String> {
        use schema::cache_roots::dsl::*;
        let root_path = self.cache_root_map.get(&root_id);
        if root_path.is_some() {
            return Some(root_path.unwrap().to_string());
        }

        // Get path from the DB and add to `cache_root_map`.
        match cache_roots
            .select(path)
            .filter(id.eq(root_id))
            .get_result::<String>(&self.connection)
        {
            Ok(root_path) => {
                self.cache_root_map.insert(root_id, root_path.to_string());
                Some(root_path)
            }
            Err(_) => None,
        }
    }

    /// Remove the cuboid's entry from the DB
    ///
    /// # Arguments
    ///
    /// * `cuboid_id` - Cuboid's id in the DB
    fn remove_cuboid_entry(&self, cuboid_id: i64) {
        use schema::cuboids::dsl::*;
        if diesel::delete(cuboids.filter(id.eq(cuboid_id)))
            .execute(&self.connection)
            .is_err()
        {
            // ToDo: write to log.
            println!("Could not delete cuboid with id: {}", cuboid_id);
        }
    }

    /// Remove the cuboid's file
    ///
    /// # Arguments
    ///
    /// * `cuboid_path` - Full path to the cuboid
    fn remove_cuboid_file(&self, cuboid_path: &str) -> std::io::Result<()> {
        let path = Path::new(cuboid_path);
        fs::remove_file(path)?;
        Ok(())
    }
}
