-- This file should undo anything in `up.sql`
DROP TRIGGER valid_user ON group_members;
DROP FUNCTION valid_user();
DROP TABLE group_members;