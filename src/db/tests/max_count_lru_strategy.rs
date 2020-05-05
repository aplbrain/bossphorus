use crate::db::models::Cuboid;
use crate::db::{LeastRecentlyUsed, LimitNumCuboids, MaxCountLruStrategy, Scheduling, Selection};
use chrono::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

struct MockLru {}

impl LeastRecentlyUsed for MockLru {
    /// Return the number of rows requested.  For these tests, row content
    /// doesn't really matter.
    fn find_lru(&self, num: u32) -> Vec<Cuboid> {
        let key = "cube";
        let rows: Vec<Cuboid> = (0..num)
            .map(|i| {
                let timestamp = Utc.ymd(2020, 4, 19).and_hms(23 - i, 0, 0).naive_utc();
                Cuboid {
                    id: (i + 1) as i64,
                    cache_root: 1,
                    cube_key: format!("{}/{}", key, i),
                    requests: i as i64,
                    created: timestamp,
                    last_accessed: timestamp,
                }
            })
            .collect();
        rows
    }
}

#[test]
fn test_ready_for_cleaning_should_be_yes() {
    let mut strat = MaxCountLruStrategy::new(100, Rc::new(RefCell::new(MockLru {})));
    strat.set_size(101);
    assert!(strat.ready_for_cleaning());
}

#[test]
fn test_ready_for_cleaning_should_be_no() {
    let mut strat = MaxCountLruStrategy::new(100, Rc::new(RefCell::new(MockLru {})));
    strat.set_size(100);
    assert_eq!(false, strat.ready_for_cleaning());
}

#[test]
fn test_sub_with_u32_overflow() {
    let mut strat = MaxCountLruStrategy::new(100, Rc::new(RefCell::new(MockLru {})));
    strat.set_size(100);
    strat.sub(102);
    assert_eq!(0, strat.size());
}

#[test]
fn test_select_cuboids_for_removal_should_be_none() {
    let mut strat = MaxCountLruStrategy::new(100, Rc::new(RefCell::new(MockLru {})));
    strat.set_size(100);
    let actual = strat.select_cuboids_for_removal();
    assert_eq!(0, actual.len());
}

#[test]
fn test_select_cuboids_for_removal() {
    let mut strat = MaxCountLruStrategy::new(100, Rc::new(RefCell::new(MockLru {})));
    strat.set_size(104);
    let actual = strat.select_cuboids_for_removal();
    assert_eq!(4, actual.len());
}