-- Add migration script here

-- Tabela Użytkowników
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    permission TEXT NOT NULL, -- (Admin, User, Guest)
    valid BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS users_data (
    username TEXT PRIMARY KEY,
    email TEXT,
    name TEXT,
    surname TEXT,
    FOREIGN KEY(username) REFERENCES users(username) ON DELETE CASCADE
);

-- Tabela Produktów
CREATE TABLE IF NOT EXISTS products (
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
CREATE TABLE IF NOT EXISTS images (
    product_id INTEGER PRIMARY KEY,
    warianty_zdjec TEXT NOT NULL, -- Tutaj leci zrzutowany JSON z rozdzielczościami
    FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Tabela Modeli 3D - pod Twoją strukturę Model z LODami
CREATE TABLE IF NOT EXISTS models (
    product_id INTEGER PRIMARY KEY,
    texture_ao TEXT, -- Usunięto UNIQUE, żeby brak tekstury (NULL) nie blokował bazy
    model TEXT NOT NULL, -- Tutaj leci zrzutowany JSON z LODami
    FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Tabela Zamówień (Nagłówek spłaszczony przez serde flatten)
CREATE TABLE IF NOT EXISTS orders (
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

    cena_dziesiatki INTEGER NOT NULL,
    cena_grosze INTEGER NOT NULL,
    vat_dziesiatki INTEGER NOT NULL DEFAULT 0,
    vat_grosze INTEGER NOT NULL DEFAULT 0,
    waluta TEXT NOT NULL,
    numer_fv TEXT NOT NULL,
    oplacone TEXT NOT NULL,
    status TEXT NOT NULL,

    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Tabela Pozycji Zamówienia
CREATE TABLE IF NOT EXISTS orders_things (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- SQLite lubi mieć jednoznaczne ID dla każdego wiersza
    zamowienie_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    ilosc INTEGER NOT NULL,
    cena_dziesiatki INTEGER NOT NULL,
    cena_grosze INTEGER NOT NULL,
    vat_dziesiatki INTEGER NOT NULL DEFAULT 0,
    vat_grosze INTEGER NOT NULL DEFAULT 0,
    waluta TEXT NOT NULL,
    konfiguracja TEXT, -- Serde json Value jako TEXT

    FOREIGN KEY(zamowienie_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE RESTRICT
);