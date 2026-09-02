import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import { userApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'
import { statusOf } from '@/lib/httpError'

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
  // updateUser replaces the whole profile; an omitted avatarUrl clears it,
  // so callers must always pass the person's current value.
  avatarUrl: string | null
  status: UserStatus
  drivingLicenses: DrivingLicense[]
  wateringPlanSelectable: boolean
}

/** 403 (role exceeds own grants) and 409 (own account) are shown at the card. */
const isFieldLevel = (error: unknown): boolean => {
  const status = statusOf(error)
  return status === 403 || status === 409
}

export const useUserMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['users'] })
  }

  const assignRole = useMutation({
    mutationFn: ({ userId, roleId }: AssignRoleVariables) =>
      userApi.assignUserRole({ userId, assignRoleRequest: { roleId } }),
    onSuccess: () => {
      invalidate()
      showToast(t('members.toast.roleAssigned'))
    },
    onError: (error) => {
      if (!isFieldLevel(error)) showToast(t('members.toast.roleAssignFailed'), 'error')
    },
  })

  const revokeRole = useMutation({
    mutationFn: ({ userId, roleId }: RevokeRoleVariables) =>
      userApi.revokeUserRole({ userId, roleId }),
    onSuccess: () => {
      invalidate()
      showToast(t('members.toast.roleRevoked'))
    },
    onError: (error) => {
      if (!isFieldLevel(error)) showToast(t('members.toast.roleRevokeFailed'), 'error')
    },
  })

  const setOrganization = useMutation({
    mutationFn: ({ userId, organizationId }: SetOrganizationVariables) =>
      userApi.setUserOrganization({ userId, setOrganizationRequest: { organizationId } }),
    onSuccess: () => {
      invalidate()
      showToast(t('members.toast.organizationChanged'))
    },
    onError: (error) => {
      if (!isFieldLevel(error)) {
        showToast(t('members.toast.organizationChangeFailed'), 'error')
      }
    },
  })

  const updateProfile = useMutation({
    mutationFn: ({
      userId,
      employeeId,
      phoneNumber,
      avatarUrl,
      status,
      drivingLicenses,
      wateringPlanSelectable,
    }: UpdateProfileVariables) =>
      userApi.updateUser({
        userId,
        userUpdateRequest: {
          employeeId,
          phoneNumber,
          avatarUrl,
          status,
          drivingLicenses,
          wateringPlanSelectable,
        },
      }),
    onSuccess: () => {
      invalidate()
      showToast(t('members.toast.profileSaved'))
    },
    onError: (error) => {
      if (!isFieldLevel(error)) showToast(t('members.toast.profileSaveFailed'), 'error')
    },
  })

  return { assignRole, revokeRole, setOrganization, updateProfile }
}
