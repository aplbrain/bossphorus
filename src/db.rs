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

/// SQL database module.
pub mod models;
pub mod schema;

extern crate chrono;
extern crate diesel;
use super::config;
use super::usage_tracker::UsageTracker;
use chrono::prelude::*;
use diesel::prelude::*;
use models::{CacheRoot, Cuboid, NewCacheRoot, NewCuboid};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::option::Option;
use std::path::Path;
use std::rc::Rc;
use std::result::Result;

#[cfg(test)]
pub mod tests;

pub trait Scheduling {
    /// Returns true if it's time to start removing cuboids from the cache.
    fn ready_for_cleaning(&self) -> bool;
}

pub trait Selection {
    /// Chooses cuboids to remove from the cache.
    fn select_cuboids_for_removal(&self) -> Vec<Cuboid>;
}

pub trait LeastRecentlyUsed {
    /// Find the `num` least recently used cuboids in the cache.  This is one
    /// selection strategy.
    ///
    /// # Arguments:
    ///
    /// * `num` - How many cuboids to retrieve.
    fn find_lru(&self, num: u32) -> Vec<Cuboid>;
}

/// A primitive way of managing the size of the cuboid cache.  Just limit the
/// maximun number of cuboids stored.
pub trait LimitNumCuboids {
    /// Get the max current number of cuboids that the cache should store.
    fn get_max_cuboids(&self) -> u32;

    /// Set the max number of cuboids that the cache should store.
    ///
    /// # Arguments:
    ///
    /// * `max` - Max number of cuboids to store.
    fn set_max_cuboids(&mut self, max: u32);

    /// Get the current number of cuboids stored in the cache.
    fn size(&self) -> u32;

    /// Set the current number of cuboids in the cache.
    ///
    /// # Arguments:
    ///
    /// * `num` - Current number of cuboids.
    fn set_size(&mut self, num: u32);

    /// Update the count by adding the given number.
    ///
    /// # Arguments:
    ///
    /// * `num` - Increment the count by this value.
    fn add(&mut self, num: u32);

    /// Update the count by subtracting the given number.  If result would
    /// be a negative number, count is set to zero.
    ///
    /// # Arguments:
    ///
    /// * `num` - Deccrement the count by this value.
    fn sub(&mut self, num: u32);
}

/// A full, but primitive, cache management strategy.  Limit the maximum
/// number of cuboids in the cache by evicting the least recently used ones.
pub struct MaxCountLruStrategy {
    /// Max number of cuboids stored in the cache.
    max_cuboids: u32,
    /// Current number of cuboids stored in the cache.
    num_cuboids: u32,
    /// Find cuboids based on least recently used.
    finder: Rc<RefCell<dyn LeastRecentlyUsed>>,
}

impl Scheduling for MaxCountLruStrategy {
    fn ready_for_cleaning(&self) -> bool {
        self.size() > self.get_max_cuboids()
    }
}

impl LimitNumCuboids for MaxCountLruStrategy {
    fn get_max_cuboids(&self) -> u32 {
        self.max_cuboids
    }

    fn set_max_cuboids(&mut self, max: u32) {
        self.max_cuboids = max;
    }

    fn size(&self) -> u32 {
        self.num_cuboids
    }

    fn set_size(&mut self, num: u32) {
        self.num_cuboids = num;
    }

    fn add(&mut self, num: u32) {
        self.num_cuboids += num;
    }

    fn sub(&mut self, num: u32) {
        if num > self.num_cuboids {
            self.num_cuboids = 0;
        } else {
            self.num_cuboids -= num;
        }
    }
}

impl Selection for MaxCountLruStrategy {
    fn select_cuboids_for_removal(&self) -> Vec<Cuboid> {
        let num_to_remove = self.size() as i64 - self.get_max_cuboids() as i64;
        if num_to_remove <= 0 {
            return Vec::<Cuboid>::new();
        }
        self.finder.borrow().find_lru(num_to_remove as u32)
    }
}

impl MaxCountLruStrategy {
    pub fn new(
        max_cuboids: u32,
        finder: Rc<RefCell<dyn LeastRecentlyUsed>>,
    ) -> MaxCountLruStrategy {
        let num_cuboids = 0;

        MaxCountLruStrategy {
            max_cuboids,
            num_cuboids,
            finder,
        }
    }
}

/// Do simple cache management with cache data backed by SQLite.
pub struct SimpleCacheManager {
    /// All DB accesses use this object.
    db: Rc<RefCell<SqliteCacheInterface>>,
    /// Cache management strategy implementation (keep no more than _n_ files; remove least recently used).
    strategy: MaxCountLruStrategy,
}

impl UsageTracker for SimpleCacheManager {
    fn log_request(&mut self, key: String) {
        if self.db.borrow_mut().log_request(key) {
            // Added a new cuboid, so check if time to start cleaning cache.
            self.strategy.add(1);
            if self.strategy.ready_for_cleaning() {
                let cuboids = self.strategy.select_cuboids_for_removal();
                let num_removed = self.db.borrow_mut().clean_cache(cuboids);
                self.strategy.sub(num_removed);
            }
        }
    }
}

impl SimpleCacheManager {
    pub fn new(
        db: Rc<RefCell<SqliteCacheInterface>>,
        strategy: MaxCountLruStrategy,
    ) -> SimpleCacheManager {
        SimpleCacheManager { db, strategy }
    }
}

/// Wrap file removal for use in testing.
trait FileRemover {
    fn remove(&self, path: &Path) -> std::io::Result<()>;
}

struct RealFileRemover {}

impl FileRemover for RealFileRemover {
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }
}
/// Provides an API for maintaining cache metadata via SQLite.
pub struct SqliteCacheInterface {
    /// The connection to the DB.
    connection: SqliteConnection,
    /// id of the cache root in the `cache_roots` table.  Need for inserts into
    /// `cuboids` table.
    cache_root_id: i32,
    /// Store all cache roots encountered during execution.
    cache_root_map: HashMap<i32, String>,
    /// The byte length of the CUBOID_ROOT_PATH.
    path_len: usize,
    /// Removes cuboids from the file system.
    file: Rc<dyn FileRemover>,
}

impl LeastRecentlyUsed for SqliteCacheInterface {
    fn find_lru(&self, num: u32) -> Vec<Cuboid> {
        use schema::cuboids::dsl::*;
        cuboids
            .order(last_accessed)
            .limit(num as i64)
            .load::<Cuboid>(&self.connection)
            .expect("Error getting LRU cuboids")
    }
}

diesel_migrations::embed_migrations!();

impl SqliteCacheInterface {
    /// Constructor.
    ///
    /// # Arguments:
    ///
    /// * `db_url` - Connection string for the Sqlite DB
    /// * `strategy` - Logic for managing size of cache.
    pub fn new(db_url: &str) -> SqliteCacheInterface {
        let connection =
            SqliteConnection::establish(db_url).expect(&format!("Error connecting to {}", db_url));
        embedded_migrations::run(&connection).expect("Error running database migrations");
        SqliteCacheInterface::init(connection, Rc::new(RealFileRemover {}))
    }

    /// Completes setup of the manager.  Called directly by the `new()` constructor.
    ///
    /// # Arguments:
    ///
    /// * `connection` - Open Sqlite connection
    /// * `file_remover` - Used to remove cuboids from the file system.
    fn init(
        connection: SqliteConnection,
        file_remover: Rc<dyn FileRemover>,
    ) -> SqliteCacheInterface {
        let cache_root_id = SqliteCacheInterface::get_cache_root_id(&connection);
        let mut cache_root_map = HashMap::new();
        cache_root_map.insert(cache_root_id, config::CUBOID_ROOT_PATH.to_string());
        let path_len = config::CUBOID_ROOT_PATH.len();
        let file = file_remover;

        return SqliteCacheInterface {
            connection,
            cache_root_id,
            cache_root_map,
            path_len,
            file,
        };
    }

    /// Remove the given list of cuboids from the cache.  Returns the number of
    /// cuboids successfully removed.
    ///
    /// # Arguments
    ///
    /// * `unwanted` - List of cuboids to remove from the cache
    pub fn clean_cache(&mut self, unwanted: Vec<Cuboid>) -> u32 {
        let mut remove_count: u32 = 0;

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
                if self.remove_cuboid_entry(cuboid.id).is_ok() {
                    remove_count += 1;
                } else {
                    // ToDo: write to log.
                    println!("Error removing {} from DB", cuboid.cube_key);
                }
            }
        }

        remove_count
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
                SqliteCacheInterface::get_cache_root_id(connection)
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

    fn log_request(&self, key: String) -> bool {
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
            Err(err) => {
                println!("Error updating DB: {}", err);
                false
            }
            Ok(num_rows) => {
                if num_rows > 0 {
                    return false;
                }
                let new_request = NewCuboid {
                    cache_root: self.cache_root_id,
                    cube_key: remainder.to_string(),
                    requests: 1,
                };
                match diesel::insert_into(cuboids)
                    .values(&new_request)
                    .execute(&self.connection)
                {
                    Ok(_) => true,
                    Err(err) => {
                        println!("insert failed: {}", err);
                        false
                    }
                }
            }
        }
    }

    /// Remove the cuboid's entry from the DB
    ///
    /// # Arguments
    ///
    /// * `cuboid_id` - Cuboid's id in the DB
    fn remove_cuboid_entry(&self, cuboid_id: i64) -> QueryResult<()> {
        use schema::cuboids::dsl::*;
        diesel::delete(cuboids.filter(id.eq(cuboid_id))).execute(&self.connection)?;
        Ok(())
    }

    /// Remove the cuboid's file
    ///
    /// # Arguments
    ///
    /// * `cuboid_path` - Full path to the cuboid
    fn remove_cuboid_file(&self, cuboid_path: &str) -> std::io::Result<()> {
        let path = Path::new(cuboid_path);
        //fs::remove_file(path)?;
        self.file.remove(path)?;
        Ok(())
    }
}
