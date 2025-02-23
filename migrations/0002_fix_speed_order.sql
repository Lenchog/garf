CREATE TABLE fixed_score (
    LayoutId INTEGER NOT NULL,
    User TEXT NOT NULL,
    Speed INTEGER NOT NULL,
    FOREIGN KEY (LayoutId) REFERENCES layout(LayoutId)
);

INSERT INTO fixed_score (LayoutId, User, Speed)
SELECT LayoutId, User, CAST(Speed AS INTEGER)
FROM score;

DROP TABLE score;
ALTER TABLE fixed_score RENAME TO score;
