-- Your SQL goes here
CREATE TABLE group_ancestors (
    group_id UUID NOT NULL,
    ancestor_group_id UUID NOT NULL,
    PRIMARY KEY (group_id, ancestor_group_id),
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (ancestor_group_id) REFERENCES groups(id)
);

CREATE FUNCTION manage_group_ancestors() RETURNS trigger AS $manage_group_ancestors$
    BEGIN
        IF NEW.parent IS NOT NULL THEN
            INSERT INTO group_ancestors(group_id, ancestor_group_id)
            SELECT NEW.id AS group_id, ancestor_group_id
            FROM group_ancestors 
            WHERE group_id = NEW.parent;
            INSERT INTO group_ancestors(group_id, ancestor_group_id)
            VALUES (NEW.id, NEW.parent);
        END IF;
        RETURN NEW;
    END;
$manage_group_ancestors$ LANGUAGE plpgsql;
    
CREATE TRIGGER manage_group_ancestors AFTER INSERT ON groups
    FOR EACH ROW EXECUTE PROCEDURE manage_group_ancestors();