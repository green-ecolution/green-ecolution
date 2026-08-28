-- One row per sensor whose flagged readings have been reviewed. A missing row
-- means never acknowledged; the timestamp is a watermark, so readings flagged
-- after it raise the warning again on their own.
CREATE TABLE sensor_quality_acknowledgements (
    sensor_id       varchar   PRIMARY KEY REFERENCES sensors (id) ON DELETE CASCADE,
    acknowledged_at timestamp NOT NULL,
    acknowledged_by uuid      NOT NULL,
    note            text
);
