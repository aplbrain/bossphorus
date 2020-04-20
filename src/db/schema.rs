table! {
    cache_roots (id) {
        id -> Integer,
        path -> Text,
    }
}

table! {
    cuboids (id) {
        id -> BigInt,
        cache_root -> Integer,
        cube_key -> Text,
        requests -> BigInt,
        created -> Timestamp,
        last_accessed -> Timestamp,
    }
}

joinable!(cuboids -> cache_roots (cache_root));

allow_tables_to_appear_in_same_query!(
    cache_roots,
    cuboids,
);
