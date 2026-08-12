-- Add migration script here
-- Dodanie kolumny z automatycznym uzupełnieniem istniejących wierszy
ALTER TABLE products ADD COLUMN plastik_qua REAL DEFAULT 0.0;

-- Dodanie kolumny z automatycznym uzupełnieniem istniejących wierszy
ALTER TABLE models ADD COLUMN texture_scale REAL DEFAULT 1.0;