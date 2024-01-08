-- This file should undo anything in `up.sql`
DROP TRIGGER delete_access_types ON ressource_access_types;
DROP FUNCTION delete_access_types;
DROP TABLE IF EXISTS ressource_access_types;