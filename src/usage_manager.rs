/// Usage management module.
///
/// Tracks usage of the cached cuboids stored locally on disk.
///
/// A single thread receives keys from the Rocket worker threads as cuboids are
/// accessed.
use super::db::SqliteCacheManager;
use std::env;
use std::sync;
use std::sync::mpsc;
use std::thread;

/// User string names for selecting usage managers.
const NONE_MANAGER: &str = "none";
const CONSOLE_MANAGER: &str = "console";
const DB_MANAGER: &str = "db";

const DB_URL_ENV_NAME: &str = "BOSSPHORUS_DB_URL";

pub enum UsageManagerType {
    None,
    Console,
    Sqlite,
}

/// Map string name of usage manager to enum.  If no match is found, return
/// UsageManagerType::None.
pub fn get_manager_type(name: &str) -> UsageManagerType {
    let lowered = name.to_lowercase();
    match lowered.as_str() {
        CONSOLE_MANAGER => UsageManagerType::Console,
        NONE_MANAGER => UsageManagerType::None,
        DB_MANAGER => UsageManagerType::Sqlite,
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
        UsageManagerType::Sqlite => {
            let db_url = env::var(DB_URL_ENV_NAME).expect(&format!(
                "{} environment variable must be set",
                &DB_URL_ENV_NAME
            ));
            Box::new(SqliteCacheManager::new(&db_url))
        }
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
///
/// # Arguments:
///
/// * `kind` - Which usage manager to start
/// * `cuboid_root_path` - Root of cached cuboids
pub fn run(kind: UsageManagerType) {
    if let UsageManagerType::None = kind {
        return;
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
