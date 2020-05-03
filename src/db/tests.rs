use crate::db::{FileRemover, SqliteCacheInterface};
use diesel::prelude::*;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub mod max_count_lru_strategy;
pub mod simple_cache_manager;
pub mod sqlite;

diesel_migrations::embed_migrations!();

struct MockFileRemover {
    /// Stores calls to the mock file remover.
    calls: Rc<RefCell<Vec<String>>>,
}

impl FileRemover for MockFileRemover {
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        match path.to_str() {
            Some(p) => self.calls.borrow_mut().push(p.to_string()),
            None => (),
        }
        Ok(())
    }
}

impl MockFileRemover {
    fn new(calls: Rc<RefCell<Vec<String>>>) -> MockFileRemover {
        MockFileRemover {
            calls,
        }
    }
}

struct SqlCacheInterfaceTestItems {
    sql_mgr: SqliteCacheInterface,
    remove_calls: Rc<RefCell<Vec<String>>>,
}

/// Set up an in-memory Sqlite DB and return a SqliteCacheManager for testing.
fn setup_db() -> SqlCacheInterfaceTestItems {
    let connection = SqliteConnection::establish(":memory:").unwrap();
    embedded_migrations::run(&connection).unwrap();
    let remove_calls = Rc::new(RefCell::new(Vec::<String>::new()));
    let clone = Rc::clone(&remove_calls);
    let sql_mgr = SqliteCacheInterface::init(connection, Rc::new(MockFileRemover::new(clone)));

    SqlCacheInterfaceTestItems { sql_mgr, remove_calls }
}
