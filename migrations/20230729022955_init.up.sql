-- Add up migration script here
CREATE TABLE `urls` (
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `path` TEXT NOT NULL,
    `title` TEXT NOT NULL,
    `body` TEXT NOT NULL,
    `icon` TEXT NOT NULL,
    `redirect` TEXT NOT NULL,
    `hits` INTEGER NOT NULL DEFAULT 0,
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX `urls_index` ON `urls` (`path`);
