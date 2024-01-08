-- Your SQL goes here
CREATE TABLE role_permissions (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    role VARCHAR(45) NOT NULL,
    ressource VARCHAR(45) NOT NULL,
    UNIQUE(role, ressource),
    FOREIGN KEY (role) REFERENCES roles(value_key),
    FOREIGN KEY (ressource) REFERENCES ressources(key_value)
);