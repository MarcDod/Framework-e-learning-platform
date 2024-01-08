-- Your SQL goes here
CREATE TABLE user_access_types (
    user_permission_id UUID NOT NULL,
    access_type ACCESS_TYPE NOT NULL,
    permission BOOLEAN DEFAULT(FALSE) NOT NULL,
    set_permission BOOLEAN DEFAULT(FALSE) NOT NULL,
    set_set_permission BOOLEAN DEFAULT(FALSE) NOT NULL,
    PRIMARY KEY (user_permission_id, access_type),
    FOREIGN KEY (user_permission_id) REFERENCES user_permissions(id) ON DELETE CASCADE
);

CREATE FUNCTION valid_access_type() RETURNS trigger AS $valid_access_type$
    DECLARE
        ressource_key_value VARCHAR(45);
        valid boolean;
    BEGIN
        ressource_key_value := (
            SELECT up.ressource 
            FROM user_permissions up
            WHERE up.id = NEW.user_permission_id
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
$valid_access_type$ LANGUAGE plpgsql;
    
CREATE TRIGGER valid_access_type BEFORE INSERT or UPDATE ON user_access_types
    FOR EACH ROW EXECUTE PROCEDURE valid_access_type();

CREATE FUNCTION valid_user_access_type() RETURNS trigger AS $valid_user_access_type$
    BEGIN
        IF NEW.permission = False
        AND NEW.set_permission = False
        AND NEW.set_set_permission = False
        THEN
            RAISE EXCEPTION 'Access_type is invalid';
        END IF;

        RETURN NEW;
    END;
$valid_user_access_type$ LANGUAGE plpgsql;

CREATE TRIGGER valid_user_access_type BEFORE INSERT ON user_access_types
    FOR EACH ROW EXECUTE PROCEDURE valid_user_access_type();

CREATE FUNCTION delete_if_all_false() RETURNS trigger AS $delete_if_all_false$
	DECLARE
		amount INT;
	BEGIN
        IF NEW.permission = False
        AND NEW.set_permission = False
        AND NEW.set_set_permission = False
        THEN
            DELETE FROM user_access_types 
            WHERE user_permission_id = NEW.user_permission_id
            AND access_type = NEW.access_type;
        END IF;

		amount := (
			SELECT COUNT(*)
			FROM user_access_types
			WHERE user_permission_id = NEW.user_permission_id
		);

		IF amount = 0 THEN
            DELETE FROM user_permissions up
            WHERE up.id = NEW.user_permission_id;
		END IF;
		
        RETURN NEW;
    END;
$delete_if_all_false$ LANGUAGE plpgsql;
    
CREATE TRIGGER delete_if_all_false AFTER UPDATE ON user_access_types
    FOR EACH ROW EXECUTE PROCEDURE delete_if_all_false();   