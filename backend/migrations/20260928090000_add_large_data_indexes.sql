-- Indexes for the read paths the benchdb campaign showed scanning whole tables.

-- The radius queries compare as geography; the plain geometry GIST index
-- cannot serve a cast column, so the expression has to match exactly.
CREATE INDEX IF NOT EXISTS idx_trees_geography
    ON trees USING GIST ((geometry::geography));

-- Serves the tree list's default order (prefix, then digits numerically, then
-- id) so the first pages no longer sort the whole table. The expressions must
-- stay identical to TREE_SORT_COLUMNS in pg_tree.rs.
CREATE INDEX IF NOT EXISTS idx_trees_number_sort
    ON trees (
        (substring(number from '^\D*')),
        (NULLIF(substring(number from '\d+'), '')::numeric),
        id
    );

-- Free-text search uses ILIKE '%term%', which only a trigram index can serve.
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS idx_trees_number_trgm
    ON trees USING GIN (number gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_trees_species_trgm
    ON trees USING GIN (species gin_trgm_ops);

-- Time-window reads (soil moisture series, reading history) filter on
-- updated_at; without it every reading of the sensor is visited.
CREATE INDEX IF NOT EXISTS idx_sensor_data_sensor_updated
    ON sensor_data (sensor_id, updated_at);
