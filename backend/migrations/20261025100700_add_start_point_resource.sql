-- Start points were guarded by watering_plan before they became a core
-- resource; every holder keeps exactly the actions it had.
UPDATE roles
SET permissions = permissions || ARRAY(
    SELECT 'start_point:' || split_part(p, ':', 2)
    FROM unnest(permissions) AS p
    WHERE p LIKE 'watering_plan:%'
      AND NOT ('start_point:' || split_part(p, ':', 2)) = ANY (permissions)
)
WHERE EXISTS (SELECT 1 FROM unnest(permissions) AS p WHERE p LIKE 'watering_plan:%');

UPDATE plugins
SET permissions = permissions || ARRAY(
    SELECT 'start_point:' || split_part(p, ':', 2)
    FROM unnest(permissions) AS p
    WHERE p LIKE 'watering_plan:%'
      AND NOT ('start_point:' || split_part(p, ':', 2)) = ANY (permissions)
)
WHERE EXISTS (SELECT 1 FROM unnest(permissions) AS p WHERE p LIKE 'watering_plan:%');
