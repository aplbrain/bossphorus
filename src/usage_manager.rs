/// Usage management module.
///
/// Tracks usage of the cached cuboids stored locally on disk.
///
/// A single thread receives keys from the Rocket worker threads as cuboids are
/// accessed.

extern crate diesel;
use super::config;
use crate::db;
use db::models::{CacheRoot, NewCacheRoot, NewCuboid};
use diesel::prelude::*;
use std::env;
use std::sync;
use std::sync::mpsc;
use std::thread;

/// User string names for usage managers.
const NONE_MANAGER: &str = "none";
const CONSOLE_MANAGER: &str = "console";
const DB_MANAGER: &str = "db";

const DB_URL_ENV_NAME: &str = "BOSSPHORUST_DB_URL";

pub enum UsageManagerType {
    None,
    Console,
    MySql,
}

/// Map string name of usage manager to enum.  If no match is found, return
/// UsageManagerType::None.
pub fn get_manager_type(name: &str) -> UsageManagerType {
    let lowered = name.to_lowercase();
    match lowered.as_str() {
        CONSOLE_MANAGER => UsageManagerType::Console,
        NONE_MANAGER => UsageManagerType::None,
        DB_MANAGER => UsageManagerType::MySql,
        _ => {
            println!("Warning, got unknown user manager: {}", name);
            UsageManagerType::None
        }
    }
}

fn usage_manager_factory(kind: UsageManagerType) -> Box<dyn UsageManager> {
    match kind {
        UsageManagerType::None => Box::new(NoneManager {}),
        UsageManagerType::Console => Box::new(ConsoleUsageManager {}),
        UsageManagerType::MySql => Box::new(MySqlUsageManager::new()),
    }
}

/// Provide shareable access to the sender for the thread responsible for
/// tracking cuboid usage.  This is kind of a kludge, but it doesn't look
/// like Rocket provides easy access to the worker threads.
static mut SENDER_MUTEX: Option<sync::Mutex<mpsc::Sender<String>>> = None;

/// Get the mutex so a thread may send a key to the usage manager.  run()
/// must have been called before this may be used.
pub fn get_sender() -> &'static sync::Mutex<mpsc::Sender<String>> {
    unsafe {
        match &SENDER_MUTEX {
            None => panic!("usage_manager.run() not called"),
            Some(mutex) => mutex,
        }
    }
}

/// Start the usage manager.  This should only be called ONCE.
pub fn run(kind: UsageManagerType) {
    if let UsageManagerType::None = kind {
        return
    }

    let (tx, rx) = mpsc::channel::<String>();
    unsafe {
        if SENDER_MUTEX.is_some() {
            panic!("run() may only be called once");
        }
        SENDER_MUTEX = Option::Some(sync::Mutex::new(tx));
    }

    thread::spawn(move || {
        let usage_mgr = usage_manager_factory(kind);
        for key in rx {
            usage_mgr.log_request(key);
        }
    });
}

pub trait UsageManager {

    /// Log request to console, file, or DB.
    fn log_request(&self, key: String);
}

/// Empty manager.
pub struct NoneManager {}

impl UsageManager for NoneManager {
    fn log_request(&self, _key: String) {}
}

/// Proof of concept manager.
pub struct ConsoleUsageManager {}

impl UsageManager for ConsoleUsageManager {
    /// Most basic manager - output to console.

    fn log_request(&self, key: String) {
        println!("Request: {}", key);
    }
}

pub struct MySqlUsageManager {
    connection: MysqlConnection,
    cache_root_id: i32,
}

impl UsageManager for MySqlUsageManager {
    fn log_request(&self, key: String) {
        use db::schema::cuboids::dsl::*;
        match diesel::update(cuboids.filter(cube_key.eq(&key)))
            .set(requests.eq(requests + 1))
            .execute(&self.connection)
        {
            Err(err) => println!("Error updating DB: {}", err),
            Ok(num_rows) => {
                if num_rows < 1 {
                    println!("Inserting {}", key);
                    let new_request = NewCuboid { cache_root: self.cache_root_id, cube_key: key, requests: 1 };
                    if diesel::insert_into(cuboids)
                        .values(&new_request)
                        .execute(&self.connection)
                        .is_err()
                    {
                        // ToDo: handle error
                        println!("insert failed");
                    }
                }
            }
        }
    }
}

impl MySqlUsageManager {
    pub fn new() -> MySqlUsageManager {
        let db_url = env::var(DB_URL_ENV_NAME)
            .expect(&format!("{} environment variable must be set", &DB_URL_ENV_NAME));
        MySqlUsageManager::connect_to(&db_url)
    }

    pub fn connect_to(db_url: &str) -> MySqlUsageManager {
        let connection = MysqlConnection::establish(db_url)
            .expect(&format!("Error connecting to {}", db_url));
        let cache_root_id = MySqlUsageManager::get_cache_root_id(&connection);
        return MySqlUsageManager { connection, cache_root_id };
    }

    fn get_cache_root_id(connection: &MysqlConnection) -> i32 {
        use db::schema::cache_roots::dsl::*;
        let row: std::result::Result<CacheRoot, diesel::result::Error> = cache_roots
            .filter(path.eq(config::get_cuboid_root_abs_path()))
            .limit(1)                   // Should be unique, but . . .
            .get_result(connection);
        match row {
            Ok(row) => row.id,
            Err(_) => {
                let row = NewCacheRoot { path: config::get_cuboid_root_abs_path() };
                diesel::insert_into(cache_roots)
                    .values(row)
                    .execute(connection)
                    .expect("Could not update MySQL database");
                MySqlUsageManager::get_cache_root_id(connection)
            },
        }
    }
}
