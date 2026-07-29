ALTER TABLE organizations
  ADD COLUMN street             TEXT,
  ADD COLUMN postal_code        TEXT,
  ADD COLUMN city               TEXT,
  ADD COLUMN contact_person_id  UUID REFERENCES user_profiles(id) ON DELETE SET NULL;

-- An address is all-or-nothing: either all three parts are present or none is.
ALTER TABLE organizations
  ADD CONSTRAINT organizations_address_complete
  CHECK (num_nulls(street, postal_code, city) IN (0, 3));

CREATE INDEX organizations_contact_person_id_idx
  ON organizations (contact_person_id);
