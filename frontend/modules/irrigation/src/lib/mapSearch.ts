import { useSearch } from '@tanstack/react-router'
import { z } from 'zod'
import { filterSearchSchema } from './filterSearchSchema'

export const irrigationMapSearchSchema = filterSearchSchema
  .pick({ wateringStatuses: true })
  .extend({
    tree: z.string().optional(),
    cluster: z.string().optional(),
    clusterId: z.string().optional(),
    q: z.string().optional().catch(undefined),
  })

export type IrrigationMapSearch = z.output<typeof irrigationMapSearchSchema>

/**
 * The core `/map` route already validated these keys with this schema (the
 * manifest's `map.searchSchema`) but types them loosely; parsing again
 * restores the type, and structural sharing keeps unchanged values stable.
 */
export const useIrrigationMapSearch = (): IrrigationMapSearch =>
  useSearch({
    from: '/_protected/map',
    select: (search) => irrigationMapSearchSchema.parse(search),
    structuralSharing: true,
  })
