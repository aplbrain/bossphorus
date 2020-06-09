/// Usage Tracker module.
///
/// Tracks usage of the cached cuboids stored locally on disk.
///
/// A single thread receives keys from the Rocket worker threads as cuboids are
/// accessed.
use super::db::{MaxCountLruStrategy, SimpleCacheManager, SqliteCacheInterface};
use super::config::{NONE_TRACKER, CONSOLE_TRACKER, DB_TRACKER, DB_URL};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync;
use std::sync::mpsc;
use std::thread;

// ToDo: make this configurable.
const DEFAULT_MAX_CUBOIDS: u32 = 1000;

pub enum UsageTrackerType {
    None,
    Console,
    Sqlite,
}

/// Map string name of usage tracker to enum.  If no match is found, return
/// UsageTrackerType::None.
pub fn get_tracker_type(name: &str) -> UsageTrackerType {
    let lowered = name.to_lowercase();
    match lowered.as_str() {
        CONSOLE_TRACKER => UsageTrackerType::Console,
        NONE_TRACKER => UsageTrackerType::None,
        DB_TRACKER => UsageTrackerType::Sqlite,
        _ => {
            println!("Warning, got unknown usage tracker: {}", name);
            UsageTrackerType::None
        }
    }
}

fn usage_tracker_factory(kind: UsageTrackerType) -> Box<dyn UsageTracker> {
    match kind {
        UsageTrackerType::None => Box::new(NoneTracker {}),
        UsageTrackerType::Console => Box::new(ConsoleUsageTracker {}),
        UsageTrackerType::Sqlite => {
            let db_interface = SqliteCacheInterface::new(DB_URL);
            let rc_db_iface = Rc::new(RefCell::new(db_interface));
            let clone = Rc::clone(&rc_db_iface);
            let strategy = MaxCountLruStrategy::new(DEFAULT_MAX_CUBOIDS, rc_db_iface);
            Box::new(SimpleCacheManager::new(clone, strategy))
        }
    }
}

/// Provide shareable access to the sender for the thread responsible for
/// tracking cuboid usage.  This is kind of a kludge, but it doesn't look
/// like Rocket provides easy access to the worker threads.
static mut SENDER_MUTEX: Option<sync::Mutex<mpsc::Sender<String>>> = None;

/// Get the mutex so a thread may send a key to the usage tracker.  run()
/// must have been called before this may be used.
pub fn get_sender() -> &'static sync::Mutex<mpsc::Sender<String>> {
    unsafe {
        match &SENDER_MUTEX {
            None => panic!("usage_tracker.run() not called"),
            Some(mutex) => mutex,
        }
    }
}

/// Start the usage tracker.  This should only be called ONCE.
///
/// # Arguments:
///
/// * `kind` - Which usage tracker to start
/// * `cuboid_root_path` - Root of cached cuboids
pub fn run(kind: UsageTrackerType) {
    if let UsageTrackerType::None = kind {
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
        let mut usage_mgr = usage_tracker_factory(kind);
        for key in rx {
            usage_mgr.log_request(key);
        }
    });
}

pub trait UsageTracker {
    /// Log request to console, file, or DB.
    fn log_request(&mut self, key: String);
}

/// Empty tracker.
pub struct NoneTracker {}

impl UsageTracker for NoneTracker {
    fn log_request(&mut self, _key: String) {}
}

/// Proof of concept tracker.
pub struct ConsoleUsageTracker {}

impl UsageTracker for ConsoleUsageTracker {
    /// Most basic tracker - output to console.

    fn log_request(&mut self, key: String) {
        println!("Request: {}", key);
    }
}
