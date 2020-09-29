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

use super::SqlCacheInterfaceTestItems;
use crate::config;
use crate::db::{LimitNumCuboids, MaxCountLruStrategy, SimpleCacheManager};
use crate::usage_tracker::UsageTracker;
use std::cell::RefCell;
use std::rc::Rc;

const MAX_COUNT: u32 = 10;

struct TestItems {
    cache_mgr: SimpleCacheManager,
    remove_calls: Rc<RefCell<Vec<String>>>,
}

fn setup() -> TestItems {
    let SqlCacheInterfaceTestItems {
        sql_mgr,
        remove_calls,
    } = super::setup_db();
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
    let TestItems {
        mut cache_mgr,
        remove_calls,
    } = setup();

    let key = "coll/exp/chan";
    let num_reqs = MAX_COUNT + 5;
    let requests: Vec<String> = (0..num_reqs)
        .map(|i| format!("{}/{}/{}", config::CUBOID_ROOT_PATH, key, i))
        .collect();

    for (i, req) in requests.iter().enumerate() {
        cache_mgr.log_request(req.to_string());
        if i >= MAX_COUNT as usize {
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
