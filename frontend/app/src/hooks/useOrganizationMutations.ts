import { useMutation, useQueryClient } from '@tanstack/react-query'
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

  // A rename also changes the organization name shown in the navigation, which
  // comes from the cached /users/me response.
  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ['organizations'] })
    void queryClient.invalidateQueries({ queryKey: ['users', 'me'] })
  }

  const createOrganization = useMutation({
    mutationFn: ({ parentId, name }: CreateOrganizationVariables) =>
      organizationApi.createOrganization({
        organizationCreateRequest: { name, parentId },
      }),
    onSuccess: () => {
      invalidate()
      showToast('Unterorganisation angelegt')
    },
    onError: (error) => {
      if (!isConflict(error)) {
        showToast('Die Unterorganisation konnte nicht angelegt werden.', 'error')
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
      invalidate()
      showToast('Organisation gespeichert')
    },
    onError: (error) => {
      if (!isFieldLevel(error)) {
        showToast('Die Organisation konnte nicht gespeichert werden.', 'error')
      }
    },
  })

  const deleteOrganization = useMutation({
    mutationFn: ({ orgId }: DeleteOrganizationVariables) =>
      organizationApi.deleteOrganization({ orgId }),
    onSuccess: () => {
      invalidate()
      showToast('Organisation gelöscht')
    },
    onError: (error) => {
      showToast(
        isConflict(error)
          ? 'Die Organisation hat noch Unterorganisationen oder zugeordnete Personen und kann nicht gelöscht werden.'
          : 'Die Organisation konnte nicht gelöscht werden.',
        'error',
      )
    },
  })

  return { createOrganization, updateOrganization, deleteOrganization }
}
