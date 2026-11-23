import { useMutation, useQueryClient } from '@tanstack/react-query'
import { moduleApi } from '@/api/backendApi'
import { modulesQuery } from '@/api/queries'

export interface UpdateModuleReleaseVariables {
  orgId: string
  key: string
  released: boolean
}

export const useUpdateModuleRelease = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ orgId, key, released }: UpdateModuleReleaseVariables) =>
      moduleApi.updateOrganizationModuleReleases({
        orgId,
        moduleReleasesUpdateRequest: { modules: { [key]: released } },
      }),
    onSettled: () => {
      // The releases and the active modules hang on the `modules` root.
      void queryClient.invalidateQueries({ queryKey: modulesQuery().queryKey })
      // A release reaches the whole subtree, so every organization's switches go stale,
      // but not the organization list or the counts.
      void queryClient.invalidateQueries({
        predicate: ({ queryKey }) => queryKey[0] === 'organizations' && queryKey[2] === 'modules',
      })
    },
  })
}
