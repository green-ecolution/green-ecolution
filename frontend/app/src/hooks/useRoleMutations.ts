import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { roleApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'
import { statusOf } from '@/lib/httpError'

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
  const { t } = useTranslation('settings')

  // A role change can alter the signed-in user's own grants, so nav and gating
  // must refetch alongside the role list.
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['roles'] })
    void queryClient.invalidateQueries({ queryKey: ['users', 'me'] })
  }

  // A 409 is a name conflict already surfaced at the field, so it must not toast.
  const isNameConflict = (error: unknown): boolean => statusOf(error) === 409

  const createRole = useMutation({
    mutationFn: ({ orgId, name, description, permissions }: CreateRoleVariables) =>
      roleApi.createRole({
        orgId,
        roleCreateRequest: { name, description, permissions },
      }),
    onSuccess: () => {
      invalidate()
      showToast(t('roles.toast.created'))
    },
    onError: (error) => {
      if (!isNameConflict(error)) showToast(t('roles.toast.saveFailed'), 'error')
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
      showToast(t('roles.toast.saved'))
    },
    onError: (error) => {
      if (!isNameConflict(error)) showToast(t('roles.toast.saveFailed'), 'error')
    },
  })

  const deleteRole = useMutation({
    mutationFn: ({ roleId }: DeleteRoleVariables) => roleApi.deleteRole({ roleId }),
    onSuccess: () => {
      invalidate()
      showToast(t('roles.toast.deleted'))
    },
    onError: () => showToast(t('roles.toast.deleteFailed'), 'error'),
  })

  return { createRole, updateRole, deleteRole }
}
