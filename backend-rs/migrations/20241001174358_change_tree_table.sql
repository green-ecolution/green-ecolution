ALTER TABLE trees
ADD COLUMN readonly BOOLEAN NOT NULL DEFAULT FALSE,
DROP height_above_sea_level,
DROP age;


