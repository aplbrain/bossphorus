table! {
    cache_roots (id) {
        id -> Integer,
        path -> Varchar,
    }
}

table! {
    cuboids (id) {
        id -> Integer,
        cache_root -> Integer,
        cube_key -> Varchar,
        requests -> Integer,
        created -> Timestamp,
        last_accessed -> Timestamp,
    }
}

joinable!(cuboids -> cache_roots (cache_root));

allow_tables_to_appear_in_same_query!(
    cache_roots,
    cuboids,
);
