-- Your SQL goes here
CREATE TABLE role_access_types (
    role_permission_id UUID NOT NULL,
    access_type ACCESS_TYPE NOT NULL,
    permission BOOLEAN DEFAULT(FALSE) NOT NULL,
    set_permission BOOLEAN DEFAULT(FALSE) NOT NULL,
    set_set_permission BOOLEAN DEFAULT(FALSE) NOT NULL,
    PRIMARY KEY (role_permission_id, access_type),
    FOREIGN KEY (role_permission_id) REFERENCES role_permissions(id) ON DELETE CASCADE
);

CREATE FUNCTION role_valid_access_type() RETURNS trigger AS $role_valid_access_type$
    DECLARE
        ressource_key_value VARCHAR(45);
        valid boolean;
    BEGIN
        ressource_key_value := (
            SELECT rp.ressource 
            FROM role_permissions rp
            WHERE rp.id = NEW.role_permission_id
        );
        valid := (EXISTS ( 
            SELECT 1 
            FROM ressources r
            INNER JOIN ressource_access_types rat ON rat.ressource = r.key_value
            WHERE rat.access_type = NEW.access_type
        ));

        IF valid = False THEN
            RAISE EXCEPTION 'Ressource dos not allow this access_type';
        END IF;

        RETURN NEW;
    END;
$role_valid_access_type$ LANGUAGE plpgsql;
    
CREATE TRIGGER role_valid_access_type BEFORE INSERT or UPDATE ON role_access_types
    FOR EACH ROW EXECUTE PROCEDURE role_valid_access_type();

CREATE FUNCTION valid_role_access_type() RETURNS trigger AS $valid_role_access_type$
    BEGIN
        IF NEW.permission = False
        AND NEW.set_permission = False
        AND NEW.set_set_permission = False
        THEN
            RAISE EXCEPTION 'Access_type is invalid';
        END IF;

        RETURN NEW;
    END;
$valid_role_access_type$ LANGUAGE plpgsql;

CREATE TRIGGER valid_role_access_type BEFORE INSERT ON role_access_types
    FOR EACH ROW EXECUTE PROCEDURE valid_role_access_type();

CREATE FUNCTION role_delete_if_all_false() RETURNS trigger AS $role_delete_if_all_false$
    BEGIN
        IF NEW.permission = False
        AND NEW.set_permission = False
        AND NEW.set_set_permission = False
        THEN
            DELETE FROM role_access_types 
            WHERE role_permission_id = NEW.role_permission_id
            AND access_type = NEW.access_type;
        END IF;

        RETURN NEW;
    END;
$role_delete_if_all_false$ LANGUAGE plpgsql;
    
CREATE TRIGGER role_delete_if_all_false AFTER UPDATE ON role_access_types
    FOR EACH ROW EXECUTE PROCEDURE role_delete_if_all_false();