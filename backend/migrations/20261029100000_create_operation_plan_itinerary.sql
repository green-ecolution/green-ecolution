-- The persisted driving order of a plan's route: depot, stops and refill
-- stations interleaved as the solver returned them. Replaces the deduplicated
-- station list; existing rows become 'station' entries without stops, which a
-- plan gets back on its next route computation.
ALTER TABLE operation_plan_station_visits RENAME TO operation_plan_itinerary;

ALTER TABLE operation_plan_itinerary
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'station',
    ADD COLUMN stop_id UUID,
    ALTER COLUMN name DROP NOT NULL,
    ALTER COLUMN latitude DROP NOT NULL,
    ALTER COLUMN longitude DROP NOT NULL,
    ALTER COLUMN geometry DROP NOT NULL;

ALTER TABLE operation_plan_itinerary ALTER COLUMN kind DROP DEFAULT;

ALTER TABLE operation_plan_itinerary
    ADD CONSTRAINT operation_plan_itinerary_kind_check
        CHECK (kind IN ('depot', 'stop', 'station')),
    ADD CONSTRAINT operation_plan_itinerary_shape_check CHECK (
        (kind = 'stop' AND stop_id IS NOT NULL
            AND name IS NULL AND latitude IS NULL AND longitude IS NULL AND geometry IS NULL)
        OR (kind <> 'stop' AND stop_id IS NULL
            AND name IS NOT NULL AND latitude IS NOT NULL AND longitude IS NOT NULL AND geometry IS NOT NULL)
    ),
    ADD CONSTRAINT operation_plan_itinerary_stop_fkey
        FOREIGN KEY (stop_id, plan_id) REFERENCES operation_plan_stops (stop_id, plan_id)
        ON DELETE CASCADE;
