-- Your SQL goes here
CREATE TABLE user_permissions (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID NOT NULL,
    ressource VARCHAR(45) NOT NULL,
    group_id UUID,
    UNIQUE NULLS NOT DISTINCT (user_id, ressource, group_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (ressource) REFERENCES ressources(key_value)
);
