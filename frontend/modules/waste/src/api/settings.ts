import { queryOptions } from '@tanstack/react-query'
import type { WasteSettingsResponse } from '@green-ecolution/backend-client'

// The manifest loads this eagerly, so the API client is imported on demand:
// importing the registry must not pin it before a test mocks it.

// Below the core settings key, so invalidating those reaches this one too.
export const wasteSettingsQueries = {
  byOrganization: (orgId: string) =>
    queryOptions<WasteSettingsResponse>({
      queryKey: ['organizations', orgId, 'settings', 'waste'],
      queryFn: () =>
        import('./client').then(({ wasteBinApi }) => wasteBinApi.getWasteSettings({ orgId })),
    }),
}
