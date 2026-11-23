import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { ModuleReleasesResponse } from '@green-ecolution/backend-client'
import { moduleApi } from '@/api/backendApi'
import { isOrganizationModulesQuery, modulesQuery, moduleReleasesQuery } from '@/api/queries'

export interface UpdateModuleReleaseVariables {
  orgId: string
  key: string
  released: boolean
}

export const useUpdateModuleRelease = ({ onError }: { onError: () => void }) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ orgId, key, released }: UpdateModuleReleaseVariables) =>
      moduleApi.updateOrganizationModuleReleases({
        orgId,
        moduleReleasesUpdateRequest: { modules: { [key]: released } },
      }),
    onSuccess: (tenant) => {
      queryClient.setQueryData<ModuleReleasesResponse>(
        moduleReleasesQuery().queryKey,
        (old) =>
          old && {
            ...old,
            tenants: old.tenants.some((entry) => entry.organizationId === tenant.organizationId)
              ? old.tenants.map((entry) =>
                  entry.organizationId === tenant.organizationId ? tenant : entry,
                )
              : [...old.tenants, tenant],
          },
      )
    },
    onError,
    // Returned so the mutation stays pending until the fresh releases are on screen.
    onSettled: () =>
      Promise.all([
        // The releases and the active modules hang on the `modules` root.
        queryClient.invalidateQueries({ queryKey: modulesQuery().queryKey }),
        // A release reaches the whole subtree, so every organization's switches go stale,
        // but not the organization list or the counts.
        queryClient.invalidateQueries({ predicate: isOrganizationModulesQuery }),
      ]),
  })
}
