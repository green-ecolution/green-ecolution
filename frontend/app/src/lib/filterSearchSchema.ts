import { z } from 'zod'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { Filters } from '@/context/FilterContext'

// Single source of truth for filter search params. Routes pick the subset
// they support so the schemas can never diverge again (GECO-133).
export const filterSearchSchema = z.object({
  wateringStatuses: z.array(z.enum(WateringStatus)).optional().catch(undefined),
  regions: z.array(z.string()).optional().catch(undefined),
  hasCluster: z.boolean().optional().catch(undefined),
  plantingYears: z.array(z.number()).optional().catch(undefined),
})

export type FilterSearch = z.infer<typeof filterSearchSchema>

// Parses instead of trusting the caller: useSearch({strict:false}) merges every
// route's search schema, so the filter keys arrive widened (e.g. string[]).
export const filtersFromSearch = (search: unknown): Filters => {
  const parsed = filterSearchSchema.parse(search)
  return {
    statusTags: parsed.wateringStatuses ?? [],
    regionTags: parsed.regions ?? [],
    hasCluster: parsed.hasCluster,
    plantingYears: parsed.plantingYears ?? [],
  }
}

export const searchFromFilters = (filters: Filters): FilterSearch => ({
  wateringStatuses:
    filters.statusTags.length > 0 ? (filters.statusTags as WateringStatus[]) : undefined,
  regions: filters.regionTags.length > 0 ? filters.regionTags : undefined,
  hasCluster: filters.hasCluster,
  plantingYears: filters.plantingYears.length > 0 ? filters.plantingYears : undefined,
})
