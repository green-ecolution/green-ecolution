-- Capacity kinds are declared by modules now; the core no longer lists them.
ALTER TABLE vehicle_capacities DROP CONSTRAINT vehicle_capacities_kind_check;
ALTER TABLE vehicle_capacities RENAME COLUMN liters TO amount;
ALTER TABLE vehicle_capacities RENAME CONSTRAINT vehicle_capacities_liters_check TO vehicle_capacities_amount_check;
ALTER TABLE depots DROP CONSTRAINT depots_replenishes_check;
