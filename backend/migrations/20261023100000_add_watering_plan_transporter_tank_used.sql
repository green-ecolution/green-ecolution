-- false: the run carries water in the trailer only; the transporter's own
-- tank stays empty. Only meaningful with a trailer (enforced by the domain).
ALTER TABLE watering_plans
    ADD COLUMN transporter_tank_used BOOLEAN NOT NULL DEFAULT true;
