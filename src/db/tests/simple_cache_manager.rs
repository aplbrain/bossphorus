use crate::config;
use crate::db::{LimitNumCuboids, MaxCountLruStrategy, SimpleCacheManager};
use crate::usage_tracker::UsageTracker;
use std::cell::RefCell;
use std::rc::Rc;
use super::{SqlCacheInterfaceTestItems};

const MAX_COUNT: u32 = 10;

struct TestItems {
    cache_mgr: SimpleCacheManager,
    remove_calls: Rc<RefCell<Vec<String>>>,
}

fn setup() -> TestItems {
    let SqlCacheInterfaceTestItems { sql_mgr, remove_calls } = super::setup_db();
    let db = Rc::new(RefCell::new(sql_mgr));
    let clone = Rc::clone(&db);
    let strat = MaxCountLruStrategy::new(MAX_COUNT, clone);
    TestItems { 
        cache_mgr: SimpleCacheManager::new(db, strat),
        remove_calls,
    }
}

#[test]
fn test_cache_management() {
    let TestItems { mut cache_mgr, remove_calls } = setup();

    let key = "coll/exp/chan";
    let num_reqs = MAX_COUNT + 5;
    let requests: Vec<String> = (0..num_reqs)
        .map(|i| {
            format!("{}/{}/{}", config::CUBOID_ROOT_PATH, key, i)
        })
        .collect();

    for (i, req) in requests.iter().enumerate() {
        cache_mgr.log_request(req.to_string());
        if i >= MAX_COUNT  as usize {
            // Once there are MAX_COUNT cuboids in the cache, then there
            // should be a removal of the oldest cuboid whenever a new one
            // gets added.
            let ind = i - MAX_COUNT as usize;
            assert_eq!(requests[ind], remove_calls.borrow()[ind]);
        }
    }

    let exp_removes = (num_reqs - MAX_COUNT) as usize;
    assert_eq!(exp_removes, remove_calls.borrow().len());
    assert_eq!(MAX_COUNT, cache_mgr.strategy.size());
}