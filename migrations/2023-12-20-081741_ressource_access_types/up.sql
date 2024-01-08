-- Your SQL goes here
CREATE TABLE ressource_access_types (
    ressource VARCHAR(45) NOT NULL,
    access_type ACCESS_TYPE NOT NULL,
    PRIMARY KEY (ressource, access_type),
    FOREIGN KEY (ressource) REFERENCES ressources(key_value) ON DELETE CASCADE
);

CREATE FUNCTION delete_access_types() RETURNS trigger AS $delete_access_types$
    BEGIN
        DELETE FROM user_access_types uat
        USING user_permissions up
        WHERE up.id = uat.user_permission_id
        AND up.ressource = OLD.ressource
        AND uat.access_type = OLD.access_type;

        RETURN OLD;
    END;
$delete_access_types$ LANGUAGE plpgsql;

CREATE TRIGGER delete_access_types AFTER DELETE ON ressource_access_types
    FOR EACH ROW EXECUTE PROCEDURE delete_access_types();