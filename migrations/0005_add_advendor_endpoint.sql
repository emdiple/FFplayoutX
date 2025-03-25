-- Add migration script here
ALTER TABLE global ADD COLUMN advendor_endpoint TEXT NOT NULL DEFAULT 'http://localhost:8008';
ALTER TABLE channels ADD COLUMN advendor_endpoint TEXT NOT NULL DEFAULT '';