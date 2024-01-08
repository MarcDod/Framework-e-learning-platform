-- This file should undo anything in `up.sql`
DROP TRIGGER manage_group_ancestors ON groups;
DROP FUNCTION manage_group_ancestors();
DROP TABLE group_ancestors;