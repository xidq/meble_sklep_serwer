-- Add migration script here

-- Tabela Użytkowników
CREATE TABLE users (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       username TEXT NOT NULL UNIQUE,
                       email TEXT,
                       name TEXT,
                       password_hash TEXT NOT NULL,
                       permission TEXT NOT NULL, -- Twoja rola (Admin, User, Guest)
                       valid BOOLEAN NOT NULL DEFAULT 0
);

-- Tabela Produktów
CREATE TABLE products (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          name_id TEXT NOT NULL UNIQUE,
                          name_pl TEXT,
                          name_en TEXT,
                          desc_pl TEXT,
                          desc_en TEXT,
                          wood_qua REAL,
                          metal_qua REAL,
                          glass_qua REAL,
                          price REAL NOT NULL,
                          width REAL,
                          height REAL,
                          depth REAL
);

-- Tabela Multimediów (Zdjęcia) - pod Twój BTreeMap z Rust
CREATE TABLE images (
                        product_id INTEGER PRIMARY KEY,
                        warianty_zdjec TEXT NOT NULL, -- Tutaj leci zrzutowany JSON z rozdzielczościami
                        FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Tabela Modeli 3D - pod Twoją strukturę Model z LODami
CREATE TABLE models (
                        product_id INTEGER PRIMARY KEY,
                        texture_ao TEXT, -- Usunięto UNIQUE, żeby brak tekstury (NULL) nie blokował bazy
                        model TEXT NOT NULL, -- Tutaj leci zrzutowany JSON z LODami
                        FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Tabela Zamówień (Nagłówek spłaszczony przez serde flatten)
CREATE TABLE orders (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER, -- NULL = zakup jako gość
                        imie TEXT NOT NULL,
                        nazwisko TEXT NOT NULL,
                        date TEXT NOT NULL,
                        email TEXT,
                        tel TEXT,

    -- Pola ze spłaszczonej struktury ZamowienieLokacja
                        ulica TEXT NOT NULL,
                        miasto TEXT NOT NULL,
                        kod_pocztowy TEXT NOT NULL,

    -- Pola ze spłaszczonej struktury ZamowienieFV
                        nazwa_firmy TEXT,
                        nip TEXT,
                        fv_ulica TEXT,
                        fv_miasto TEXT,
                        fv_kod_pocztowy TEXT,

    -- Pola ze spłaszczonej struktury DaneTransportu (NOWE całe)
                        odleglosc_km REAL, --n
                        cena_netto REAL,--n
                        transport_stawka_vat REAL,--n

                        cena REAL NOT NULL,
                        vat REAL NOT NULL DEFAULT 0.0, -- (NOWE)
                        numer_fv TEXT NOT NULL,
                        oplacone BOOLEAN NOT NULL DEFAULT 0,

                        FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Tabela Pozycji Zamówienia
CREATE TABLE orders_things (
                               id INTEGER PRIMARY KEY AUTOINCREMENT, -- SQLite lubi mieć jednoznaczne ID dla każdego wiersza
                               zamowienie_id INTEGER NOT NULL,
                               product_id INTEGER NOT NULL,
                               ilosc INTEGER NOT NULL,
                               cena REAL NOT NULL, -- Zamrożona cena z dnia zakupu
                               vat REAL NOT NULL DEFAULT 0.0, -- (NOWE) Stawka VAT dla konkretnej pozycji
                               konfiguracja TEXT, -- Serde json Value jako TEXT

                               FOREIGN KEY(zamowienie_id) REFERENCES orders(id) ON DELETE CASCADE,
                               FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE RESTRICT
);