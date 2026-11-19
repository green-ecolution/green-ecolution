import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { AddressDto } from '@green-ecolution/backend-client'
import { organizationApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'
import { statusOf } from '@/lib/httpError'

export interface CreateOrganizationVariables {
  parentId: string
  name: string
}

export interface UpdateOrganizationVariables {
  orgId: string
  name: string
  address: AddressDto | null
  contactPersonId: string | null
}

export interface DeleteOrganizationVariables {
  orgId: string
}

const isConflict = (error: unknown): boolean => statusOf(error) === 409

/**
 * 409 is a sibling name conflict, 422 a contact person outside the organization.
 * Both are surfaced at the field they belong to, so neither may raise a toast.
 */
const isFieldLevel = (error: unknown): boolean => {
  const status = statusOf(error)
  return status === 409 || status === 422
}

export const useOrganizationMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  // A rename also changes the organization name shown in the navigation, which
  // comes from the cached /users/me response; ['users', 'me'] also covers the
  // reachable organizations the settings resolve their organization from.
  // `deletedId`: its own detail and settings would only refetch into a 404 while
  // its page is still open.
  const invalidate = (deletedId?: string) =>
    Promise.all([
      queryClient.invalidateQueries({
        queryKey: ['organizations'],
        predicate: (query) => deletedId === undefined || query.queryKey[1] !== deletedId,
      }),
      queryClient.invalidateQueries({ queryKey: ['users', 'me'] }),
    ])

  const createOrganization = useMutation({
    mutationFn: ({ parentId, name }: CreateOrganizationVariables) =>
      organizationApi.createOrganization({
        organizationCreateRequest: { name, parentId },
      }),
    // Returning the refetch holds the caller's onSuccess, which navigates to the
    // new organization, until the reachable list knows it.
    onSuccess: () => {
      showToast(t('organization.toast.created'))
      return invalidate()
    },
    onError: (error) => {
      if (!isConflict(error)) {
        showToast(t('organization.toast.createFailed'), 'error')
      }
    },
  })

  const updateOrganization = useMutation({
    mutationFn: ({ orgId, name, address, contactPersonId }: UpdateOrganizationVariables) =>
      organizationApi.updateOrganization({
        orgId,
        organizationUpdateRequest: { name, address, contactPersonId },
      }),
    onSuccess: () => {
      void invalidate()
      showToast(t('organization.toast.saved'))
    },
    onError: (error) => {
      if (!isFieldLevel(error)) {
        showToast(t('organization.toast.saveFailed'), 'error')
      }
    },
  })

  const deleteOrganization = useMutation({
    mutationFn: ({ orgId }: DeleteOrganizationVariables) =>
      organizationApi.deleteOrganization({ orgId }),
    // Held like create: the caller then navigates on a list without the deleted one.
    onSuccess: (_, { orgId }) => {
      showToast(t('organization.toast.deleted'))
      return invalidate(orgId)
    },
    onError: (error) => {
      showToast(
        t(
          isConflict(error)
            ? 'organization.toast.deleteConflict'
            : 'organization.toast.deleteFailed',
        ),
        'error',
      )
    },
  })

  return { createOrganization, updateOrganization, deleteOrganization }
}
