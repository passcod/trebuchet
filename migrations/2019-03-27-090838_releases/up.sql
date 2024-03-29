CREATE TYPE release_state AS ENUM (
    'todo',
    'building',
    'ready'
);

CREATE TABLE releases (
    id int GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    app_id int REFERENCES apps(id),
    tag text NOT NULL,
    created timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    repo text NOT NULL,
    build_script text NOT NULL,
    state release_state NOT NULL DEFAULT 'todo'::release_state
);

CREATE TRIGGER update_timestamp
BEFORE UPDATE
ON releases
FOR EACH ROW EXECUTE FUNCTION update_timestamp();
