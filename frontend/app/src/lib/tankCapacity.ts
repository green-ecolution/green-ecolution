interface Tank {
  waterCapacity: number
}

/** Mirrors `routing::tank_capacity` in the backend domain. */
export const tankCapacity = (
  transporter: Tank,
  trailer: Tank | undefined,
  transporterTankUsed: boolean,
) => (transporterTankUsed ? transporter.waterCapacity : 0) + (trailer?.waterCapacity ?? 0)
