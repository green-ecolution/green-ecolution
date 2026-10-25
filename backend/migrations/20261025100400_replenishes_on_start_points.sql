ALTER TABLE depots
    ADD COLUMN replenishes TEXT[] NOT NULL DEFAULT '{}'
        CHECK (replenishes <@ ARRAY['water', 'cargo_volume']::TEXT[]);

UPDATE depots SET replenishes = ARRAY['water'] WHERE watering_point;

ALTER TABLE depots DROP COLUMN watering_point;
