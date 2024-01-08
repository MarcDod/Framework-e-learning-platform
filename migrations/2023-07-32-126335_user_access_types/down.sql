-- This file should undo anything in `up.sql`
DROP TRIGGER delete_if_all_false ON user_access_types;
DROP FUNCTION delete_if_all_false;
DROP TRIGGER valid_user_access_type ON user_access_types;
DROP FUNCTION valid_user_access_type;
DROP TRIGGER valid_access_type ON user_access_types;
DROP FUNCTION valid_access_type;
DROP TABLE user_access_types;