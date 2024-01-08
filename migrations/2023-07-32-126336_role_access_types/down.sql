-- This file should undo anything in `up.sql`
DROP TRIGGER role_delete_if_all_false ON role_access_types;
DROP FUNCTION role_delete_if_all_false;
DROP TRIGGER valid_role_access_type ON role_access_types;
DROP FUNCTION valid_role_access_type;
DROP TRIGGER role_valid_access_type ON role_access_types;
DROP FUNCTION role_valid_access_type;
DROP TABLE IF EXISTS role_access_types;