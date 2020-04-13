CREATE TABLE cache_roots (
    id INT AUTO_INCREMENT PRIMARY KEY,
    path VARCHAR(512) NOT NULL UNIQUE
);

CREATE TABLE cuboids (
    id INT AUTO_INCREMENT PRIMARY KEY,
    cache_root INT NOT NULL,
    cube_key VARCHAR(512) NOT NULL,
    requests INT NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    INDEX (cache_root),
    FOREIGN KEY (cache_root)
        REFERENCES cache_roots(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    INDEX (cube_key, cache_root)
);
