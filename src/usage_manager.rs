/// Usage management module.
///
/// Tracks usage of the cached cuboids stored locally on disk.
///
/// A single thread receives keys from the Rocket worker threads as cuboids are
/// accessed.
use std::sync;
use std::sync::mpsc;
use std::thread;

/// User string names for usage managers.
const NONE_MANAGER: &str = "none";
const CONSOLE_MANAGER: &str = "console";

pub enum UsageManagerType {
    None,
    Console,
}

/// Map string name of usage manager to enum.  If no match is found, return
/// UsageManagerType::None.
pub fn get_manager_type(name: &str) -> UsageManagerType {
    let lowered = name.to_lowercase();
    match lowered.as_str() {
        CONSOLE_MANAGER => UsageManagerType::Console,
        NONE_MANAGER => UsageManagerType::None,
        _ => {
            println!("Warning, got unknown user manager: {}", name);
            UsageManagerType::None
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
pub fn run() {
    let (tx, rx) = mpsc::channel::<String>();
    unsafe {
        if SENDER_MUTEX.is_some() {
            panic!("run() may only be called once");
        }
        SENDER_MUTEX = Option::Some(sync::Mutex::new(tx));
    }

    // ToDo: make manager configurable.
    thread::spawn(move || {
        let usage_mgr = ConsoleUsageManager {};
        for key in rx {
            usage_mgr.log_request(key);
        }
    });
}

pub trait UsageManager {

    /// Log request to console, file, or DB.
    fn log_request(&self, key: String);
}

/// Proof of concept manager.
pub struct ConsoleUsageManager {}

impl UsageManager for ConsoleUsageManager {
    /// Most basic manager - output to console.

    fn log_request(&self, key: String) {
        println!("Request: {}", key);
    }
}

// ToDo: create DB manager that tracks keys along with number of accesses,
// timestamp of first acces, timestamp of last access.
