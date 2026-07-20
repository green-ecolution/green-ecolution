CREATE TYPE user_status AS ENUM ('available', 'absent');

CREATE TABLE user_profiles (
    id               UUID PRIMARY KEY,
    employee_id      TEXT,
    phone_number     TEXT,
    avatar_url       TEXT,
    status           user_status NOT NULL DEFAULT 'available',
    driving_licenses driving_license[] NOT NULL DEFAULT '{}',
    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_user_profiles_updated_at
BEFORE UPDATE ON user_profiles
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
