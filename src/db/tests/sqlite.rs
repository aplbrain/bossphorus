use crate::db::models::Cuboid;
use crate::db::{LeastRecentlyUsed, schema};
use crate::config;
use chrono::prelude::*;
use diesel::prelude::*;
use super::{SqlCacheInterfaceTestItems};

#[test]
fn test_log_new_request() {
    use schema::cuboids::dsl::*;

    let SqlCacheInterfaceTestItems { sql_mgr, .. } = super::setup_db();
    let key = "/new_key";
    let actual = sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key));
    assert_eq!(true, actual);
    assert_eq!(
        Ok((key.to_string(), 1)),
        cuboids
            .select((cube_key, requests))
            .filter(cube_key.eq(key))
            .first::<(String, i64)>(&sql_mgr.connection)
    );
}

#[test]
fn test_log_repeated_request() {
    use schema::cuboids::dsl::*;

    let SqlCacheInterfaceTestItems { sql_mgr, .. } = super::setup_db();
    let key = "/my_key";
    assert_eq!(true, sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key)));
    assert_eq!(false, sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key)));
    assert_eq!(
        Ok((key.to_string(), 2)),
        cuboids
            .select((cube_key, requests))
            .filter(cube_key.eq(key))
            .first::<(String, i64)>(&sql_mgr.connection)
    );
}

#[test]
fn test_find_lru() {
    use schema::cuboids::dsl::*;

    let SqlCacheInterfaceTestItems { sql_mgr, .. } = super::setup_db();
    let key = "/my_key";
    let rows_to_get = 2;
    let rows_to_insert = 4;
    let exp_rows: Vec<Cuboid> = (0..rows_to_insert)
        .map(|i| {
            // Generate rows from most recently accessed to least.
            let timestamp = Utc.ymd(2020, 4, 19).and_hms(23 - i, 0, 0).naive_utc();
            Cuboid {
                id: (i + 1) as i64,
                cache_root: sql_mgr.cache_root_id,
                cube_key: format!("{}/{}", key, i),
                requests: i as i64,
                created: timestamp,
                last_accessed: timestamp,
            }
        })
        .collect();

    for row in &exp_rows {
        diesel::insert_into(cuboids)
            .values(row)
            .execute(&sql_mgr.connection)
            .unwrap();
    }

    let actual = sql_mgr.find_lru(rows_to_get);

    assert_eq!(rows_to_get as usize, actual.len());

    // Rows returned show be equal to the last n rows of `exp_rows`.
    for row in exp_rows.iter().rev().zip(actual.iter()) {
        assert_eq!(row.0, row.1);
    }
}

#[test]
fn test_get_cache_root_path_from_map_new_lookup() {
    use schema::cache_roots::dsl::*;

    let SqlCacheInterfaceTestItems { mut sql_mgr, .. } = super::setup_db();
    let root = "/some/folder";
    let cache_root_id = 100;
    diesel::insert_into(cache_roots)
        .values(&(id.eq(cache_root_id), path.eq(root)))
        .execute(&sql_mgr.connection)
        .expect("Could not add cache root");
    let actual = sql_mgr.get_cache_root_path_from_map(cache_root_id).unwrap();
    assert_eq!(root, actual);
}

#[test]
fn test_get_cache_root_path_from_map_existing_lookup() {
    let SqlCacheInterfaceTestItems { mut sql_mgr, .. } = super::setup_db();
    let cache_root_id = 100;
    let root_path = "/some/other/folder";
    sql_mgr
        .cache_root_map
        .insert(cache_root_id, root_path.to_string());
    assert_eq!(
        root_path,
        sql_mgr.get_cache_root_path_from_map(cache_root_id).unwrap()
    );
}

#[test]
fn test_remove_cuboid_entry() {
    use schema::cuboids::dsl::*;

    let SqlCacheInterfaceTestItems { sql_mgr, .. } = super::setup_db();
    let key = "/new_key";
    assert_eq!(true, sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key)));

    let row = sql_mgr.find_lru(1);

    assert!(sql_mgr.remove_cuboid_entry(row[0].id).is_ok());

    let results = cuboids
        .select(id)
        .filter(id.eq(row[0].id))
        .first::<i64>(&sql_mgr.connection);
    assert_eq!(false, results.is_ok());
}

#[test]
fn test_clean_cache() {
    let SqlCacheInterfaceTestItems { mut sql_mgr, remove_calls } = super::setup_db();
    let key1 = "/oldest_key";
    let full_key1 = format!("{}{}", config::CUBOID_ROOT_PATH, key1);
    assert_eq!(true, sql_mgr.log_request(full_key1.to_string()));

    let key2 = "/not_as_old_key";
    assert_eq!(true, sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key2)));

    let row = sql_mgr.find_lru(1);
    assert_eq!(key1, row[0].cube_key);

    let count = sql_mgr.clean_cache(row);
    assert_eq!(1, count);
    assert_eq!(1, remove_calls.borrow().len());
    assert_eq!(full_key1, remove_calls.borrow()[0]);
}