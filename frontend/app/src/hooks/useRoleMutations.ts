import { useMutation, useQueryClient } from '@tanstack/react-query'
import { roleApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'

export interface CreateRoleVariables {
  orgId: string
  name: string
  description: string | null
  permissions: string[]
}

export interface UpdateRoleVariables {
  roleId: string
  name: string
  description: string | null
  permissions: string[]
}

export interface DeleteRoleVariables {
  roleId: string
}

export const useRoleMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()

  // A role change can alter the signed-in user's own grants, so nav and gating
  // must refetch alongside the role list.
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['roles'] })
    void queryClient.invalidateQueries({ queryKey: ['users', 'me'] })
  }

  const createRole = useMutation({
    mutationFn: ({ orgId, name, description, permissions }: CreateRoleVariables) =>
      roleApi.createRole({
        orgId,
        roleCreateRequest: { name, description, permissions },
      }),
    onSuccess: () => {
      invalidate()
      showToast('Rolle angelegt')
    },
  })

  const updateRole = useMutation({
    mutationFn: ({ roleId, name, description, permissions }: UpdateRoleVariables) =>
      roleApi.updateRole({
        roleId,
        roleUpdateRequest: { name, description, permissions },
      }),
    onSuccess: () => {
      invalidate()
      showToast('Gespeichert')
    },
  })

  const deleteRole = useMutation({
    mutationFn: ({ roleId }: DeleteRoleVariables) => roleApi.deleteRole({ roleId }),
    onSuccess: () => {
      invalidate()
      showToast('Rolle gelöscht')
    },
  })

  return { createRole, updateRole, deleteRole }
}
