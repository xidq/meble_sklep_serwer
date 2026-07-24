-- Add migration script here
-- nowa tabela z rozszerzonymi danymi
CREATE TABLE users_data (
                            username TEXT PRIMARY KEY,
                            email TEXT,
                            name TEXT,
                            FOREIGN KEY(username) REFERENCES users(username) ON DELETE CASCADE
);

-- Kopiowanie istniejących danych ze starej tabeli
INSERT INTO users_data (username, email, name)
SELECT username, email, name FROM users;

-- Usuwanie przeniesionych kolumn z org tabeli users
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users DROP COLUMN name;