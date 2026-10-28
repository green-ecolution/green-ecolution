import type { QueryKey } from '@tanstack/react-query'

/** Every key prefix the waste aggregates own; invalidate through this table. */
export const wasteRoots = {
  bin: [['waste-bins']],
  settings: [['waste-settings']],
} as const satisfies Record<string, readonly QueryKey[]>
