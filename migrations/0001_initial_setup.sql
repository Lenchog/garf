-- migrations/0001_initial_setup.sql
CREATE TABLE IF NOT EXISTS layout (
    LayoutId INTEGER PRIMARY KEY AUTOINCREMENT,
    Name TEXT NOT NULL,
    Creator TEXT NOT NULL,
    Magic BOOLEAN NOT NULL,
    ThumbAlpha BOOLEAN NOT NULL,
    Focus TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS score (
    LayoutId INTEGER NOT NULL,
    User TEXT NOT NULL,
    Speed INTEGER NOT NULL,
    FOREIGN KEY (LayoutId) REFERENCES layout (LayoutId)
);
