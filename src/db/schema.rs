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

allow_tables_to_appear_in_same_query!(cache_roots, cuboids,);
