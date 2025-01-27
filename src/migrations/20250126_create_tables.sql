CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    name STRING
);

CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    n_left INTEGER,
    n_right INTEGER,
    operation INTEGER,
    status INTEGER
);


CREATE TABLE IF NOT EXISTS settings (
    user_id INTEGER PRIMARY KEY,
    plus_prob FLOAT,
    plus_from INTEGER,
    plus_to INTEGER,
    minus_prob FLOAT,
    minus_from INTEGER,
    minus_to INTEGER,
    mul_prob FLOAT,
    mul_from INTEGER,
    mul_to INTEGER,
    div_prob FLOAT,
    div_from INTEGER,
    div_to INTEGER
);
