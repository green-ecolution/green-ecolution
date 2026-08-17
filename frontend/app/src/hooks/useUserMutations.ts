import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import { userApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'

export interface AssignRoleVariables {
  userId: string
  roleId: string
}

export interface RevokeRoleVariables {
  userId: string
  roleId: string
}

export interface SetOrganizationVariables {
  userId: string
  organizationId: string
}

export interface UpdateProfileVariables {
  userId: string
  employeeId: string | null
  phoneNumber: string | null
  status: UserStatus
  drivingLicenses: DrivingLicense[]
}

const statusOf = (error: unknown): number | undefined =>
  (error as { response?: { status?: number } } | null)?.response?.status

/** 403 (role exceeds own grants) and 409 (own account) are shown at the card. */
const isFieldLevel = (error: unknown): boolean => {
  const status = statusOf(error)
  return status === 403 || status === 409
}

export const useUserMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()

  // A role or organization change can alter the signed-in user's own grants,
  // so nav and gating must refetch alongside the user list.
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['users'] })
    void queryClient.invalidateQueries({ queryKey: ['users', 'me'] })
  }

  const assignRole = useMutation({
    mutationFn: ({ userId, roleId }: AssignRoleVariables) =>
      userApi.assignUserRole({ userId, assignRoleRequest: { roleId } }),
    onSuccess: () => {
      invalidate()
      showToast('Rolle zugewiesen')
    },
    onError: (error) => {
      if (!isFieldLevel(error)) showToast('Die Rolle konnte nicht zugewiesen werden.', 'error')
    },
  })

  const revokeRole = useMutation({
    mutationFn: ({ userId, roleId }: RevokeRoleVariables) =>
      userApi.revokeUserRole({ userId, roleId }),
    onSuccess: () => {
      invalidate()
      showToast('Rolle entzogen')
    },
    onError: (error) => {
      if (!isFieldLevel(error)) showToast('Die Rolle konnte nicht entzogen werden.', 'error')
    },
  })

  const setOrganization = useMutation({
    mutationFn: ({ userId, organizationId }: SetOrganizationVariables) =>
      userApi.setUserOrganization({ userId, setOrganizationRequest: { organizationId } }),
    onSuccess: () => {
      invalidate()
      showToast('Organisation geändert')
    },
    onError: (error) => {
      if (!isFieldLevel(error)) {
        showToast('Die Organisation konnte nicht geändert werden.', 'error')
      }
    },
  })

  const updateProfile = useMutation({
    mutationFn: ({
      userId,
      employeeId,
      phoneNumber,
      status,
      drivingLicenses,
    }: UpdateProfileVariables) =>
      userApi.updateUser({
        userId,
        userUpdateRequest: { employeeId, phoneNumber, status, drivingLicenses },
      }),
    onSuccess: () => {
      invalidate()
      showToast('Gespeichert')
    },
    onError: (error) => {
      if (!isFieldLevel(error)) showToast('Das Profil konnte nicht gespeichert werden.', 'error')
    },
  })

  return { assignRole, revokeRole, setOrganization, updateProfile }
}
