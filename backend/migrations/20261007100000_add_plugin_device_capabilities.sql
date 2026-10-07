-- Browser features the operator granted a plugin's view. The frontend turns
-- them into the iframe's allow attribute; nothing else reads them.
ALTER TABLE plugins
    ADD COLUMN device_capabilities TEXT[] NOT NULL DEFAULT '{}'
        CONSTRAINT plugins_device_capabilities_known
        CHECK (device_capabilities <@ ARRAY['camera', 'bluetooth']::TEXT[]);
