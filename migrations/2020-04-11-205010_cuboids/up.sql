CREATE TABLE cache_roots (
    id INTEGER PRIMARY KEY NOT NULL,
    path VARCHAR(512) NOT NULL UNIQUE
);

CREATE TABLE cuboids (
    id INTEGER PRIMARY KEY NOT NULL,
    cache_root INT NOT NULL,
    cube_key VARCHAR(512) NOT NULL,
    requests BIGINT NOT NULL,
    created TEXT DEFAULT CURRENT_TIMESTAMP,
    last_accessed TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cache_root)
        REFERENCES cache_roots(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);

CREATE INDEX cuboids_cache_root_index ON cuboids(cache_root);

CREATE UNIQUE INDEX cuboids_key_cache_root_index on cuboids (cube_key, cache_root);
