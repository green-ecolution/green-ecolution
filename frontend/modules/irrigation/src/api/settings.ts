import { queryOptions } from '@tanstack/react-query'
import type { IrrigationSettingsResponse } from '@green-ecolution/backend-client'

// The manifest loads this eagerly, so the API client is imported on demand:
// importing the registry must not pin it before a test mocks it.

// Below the core settings key, so invalidating those reaches this one too.
export const irrigationSettingsQueries = {
  byOrganization: (orgId: string) =>
    queryOptions<IrrigationSettingsResponse>({
      queryKey: ['organizations', orgId, 'settings', 'irrigation'],
      queryFn: () =>
        import('./client').then(({ irrigationApi }) =>
          irrigationApi.getIrrigationSettings({ orgId }),
        ),
    }),
}
