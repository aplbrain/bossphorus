use super::models::Cuboid;
use crate::config;
use crate::usage_manager::{LeastRecentlyUsed, SqliteUsageManager, UsageManager};
use chrono::prelude::*;
use diesel::prelude::*;

diesel_migrations::embed_migrations!();

/// Set up an in-memory Sqlite DB and return a SqliteUsageManager for testing.
fn setup_db() -> SqliteUsageManager {
    let connection = SqliteConnection::establish(":memory:").unwrap();
    embedded_migrations::run(&connection).unwrap();
    SqliteUsageManager::init(connection)
}

#[test]
fn test_log_new_request() {
    use super::schema::cuboids::dsl::*;

    let sql_mgr = setup_db();
    let key = "/new_key";
    sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key));
    assert_eq!(
        Ok((key.to_string(), 1)),
        cuboids
            .select((cube_key, requests))
            .filter(cube_key.eq(key))
            .first::<(String, i64)>(sql_mgr.get_connection())
    );
}

#[test]
fn test_log_repeated_request() {
    use super::schema::cuboids::dsl::*;

    let sql_mgr = setup_db();
    let key = "/my_key";
    sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key));
    sql_mgr.log_request(format!("{}{}", config::CUBOID_ROOT_PATH, key));
    assert_eq!(
        Ok((key.to_string(), 2)),
        cuboids
            .select((cube_key, requests))
            .filter(cube_key.eq(key))
            .first::<(String, i64)>(sql_mgr.get_connection())
    );
}

#[test]
fn test_find_lru() {
    use super::schema::cuboids::dsl::*;

    let sql_mgr = setup_db();
    let key = "/my_key";
    let rows_to_get = 2;
    let rows_to_insert = 4;
    let exp_rows: Vec<Cuboid> = (0..rows_to_insert)
        .map(|i| {
            // Generate rows from most recently accessed to least.
            let timestamp = Utc.ymd(2020, 4, 19).and_hms(23 - i, 0, 0).naive_utc();
            Cuboid {
                id: (i + 1) as i64,
                cache_root: sql_mgr.get_cache_root(),
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
            .execute(sql_mgr.get_connection())
            .unwrap();
    }

    let actual = sql_mgr.find_lru(rows_to_get);

    // Rows returned show be equal to the last n rows of `exp_rows`.
    for row in exp_rows.iter().rev().zip(actual.iter()) {
        assert_eq!(row.0, row.1);
    }
}
