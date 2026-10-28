import { useSearch } from '@tanstack/react-router'
import { z } from 'zod'

export const wasteMapSearchSchema = z.object({
  wasteBin: z.string().optional(),
})

export type WasteMapSearch = z.output<typeof wasteMapSearchSchema>

/** The core `/map` route validated this key already but types it loosely; parse again to restore the type. */
export const useWasteMapSearch = (): WasteMapSearch =>
  useSearch({
    from: '/_protected/map',
    select: (search) => wasteMapSearchSchema.parse(search),
    structuralSharing: true,
  })
