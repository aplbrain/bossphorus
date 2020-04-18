use crate::config;
use crate::usage_manager::{SqliteUsageManager, UsageManager};
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
