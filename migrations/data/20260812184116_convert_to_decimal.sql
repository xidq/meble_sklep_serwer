PRAGMA foreign_keys=OFF;

-- 1. Przebudowa tabeli orders
CREATE TABLE orders_new (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            user_id INTEGER,
                            imie TEXT NOT NULL,
                            nazwisko TEXT NOT NULL,
                            date TEXT NOT NULL,
                            email TEXT,
                            tel TEXT,

                            ulica TEXT NOT NULL,
                            miasto TEXT NOT NULL,
                            kod_pocztowy TEXT NOT NULL,

                            nazwa_firmy TEXT,
                            nip TEXT,
                            fv_ulica TEXT,
                            fv_miasto TEXT,
                            fv_kod_pocztowy TEXT,

                            odleglosc_km REAL,
                            cena_netto TEXT,
                            transport_stawka_vat TEXT,

                            cena TEXT NOT NULL,
                            vat TEXT NOT NULL,
                            waluta TEXT NOT NULL,
                            numer_fv TEXT NOT NULL,
                            oplacone TEXT NOT NULL,
                            status TEXT NOT NULL,

                            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Przepisanie danych ze starych kolumn na format '00.00' dla Decimal
INSERT INTO orders_new (
    id, user_id, imie, nazwisko, date, email, tel,
    ulica, miasto, kod_pocztowy,
    nazwa_firmy, nip, fv_ulica, fv_miasto, fv_kod_pocztowy,
    odleglosc_km, cena_netto, transport_stawka_vat,
    cena, vat, waluta, numer_fv, oplacone, status
)
SELECT
    id, user_id, imie, nazwisko, date, email, tel,
    ulica, miasto, kod_pocztowy,
    nazwa_firmy, nip, fv_ulica, fv_miasto, fv_kod_pocztowy,
    odleglosc_km,
    CAST(cena_netto AS TEXT),
    CAST(transport_stawka_vat AS TEXT),
    PRINTF('%d.%02d', cena_dziesiatki, cena_grosze),
    PRINTF('%d.%02d', vat_dziesiatki, vat_grosze),
    waluta, numer_fv, oplacone, status
FROM orders;

DROP TABLE orders;
ALTER TABLE orders_new RENAME TO orders;

-- 2. Przebudowa tabeli orders_things
CREATE TABLE orders_things_new (
                                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                                   zamowienie_id INTEGER NOT NULL,
                                   product_id INTEGER NOT NULL,
                                   ilosc INTEGER NOT NULL,
                                   cena TEXT NOT NULL,
                                   vat TEXT NOT NULL,
                                   konfiguracja TEXT,

                                   FOREIGN KEY(zamowienie_id) REFERENCES orders(id) ON DELETE CASCADE,
                                   FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE RESTRICT
);

-- Przepisanie danych dla pozycji zamówienia
INSERT INTO orders_things_new (
    id, zamowienie_id, product_id, ilosc, cena, vat, konfiguracja
)
SELECT
    id, zamowienie_id, product_id, ilosc,
    PRINTF('%d.%02d', cena_dziesiatki, cena_grosze),
    PRINTF('%d.%02d', vat_dziesiatki, vat_grosze),
    konfiguracja
FROM orders_things;

DROP TABLE orders_things;
ALTER TABLE orders_things_new RENAME TO orders_things;

PRAGMA foreign_keys=ON;