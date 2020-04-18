table! {
    cache_roots (id) {
        id -> Integer,
        path -> Text,
    }
}

table! {
    cuboids (id) {
        id -> Integer,
        cache_root -> Integer,
        cube_key -> Text,
        requests -> BigInt,
        created -> Nullable<Text>,
        last_accessed -> Nullable<Text>,
    }
}

joinable!(cuboids -> cache_roots (cache_root));

allow_tables_to_appear_in_same_query!(
    cache_roots,
    cuboids,
);
