-- Your SQL goes here
CREATE TABLE group_members (
    id UUID  DEFAULT uuid_generate_v4() PRIMARY KEY,
    group_id UUID NOT NULL,
    user_id UUID NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(user_id, group_id)
);

CREATE FUNCTION valid_user() RETURNS trigger AS $valid_user$
    DECLARE
        user_state state;
    BEGIN
        user_state := (SELECT state FROM users WHERE id = NEW.user_id);

        IF user_state <> 'active' THEN
            RAISE EXCEPTION 'user is not active';
        END IF;

        RETURN NEW;
    END;
$valid_user$ LANGUAGE plpgsql;

CREATE TRIGGER valid_user BEFORE INSERT or UPDATE ON group_members
    FOR EACH ROW EXECUTE PROCEDURE valid_user();