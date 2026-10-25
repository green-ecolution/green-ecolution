interface Tank {
  capacities: Record<string, number>
}

/** Mirrors `capacity::route_capacity` for water in the backend domain. */
export const tankCapacity = (
  transporter: Tank,
  trailer: Tank | undefined,
  transporterTankUsed: boolean,
) =>
  (transporterTankUsed ? (transporter.capacities.water ?? 0) : 0) + (trailer?.capacities.water ?? 0)
