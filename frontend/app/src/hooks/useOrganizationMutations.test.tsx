import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { useQuery } from '@tanstack/react-query'
import { userQueries } from '@/api/queries'
import { renderHookWithClient } from '@/test/utils'
import { useOrganizationMutations } from './useOrganizationMutations'

const createOrganization = vi.fn()
const updateOrganization = vi.fn()
const deleteOrganization = vi.fn()
const listMyOrganizations = vi.fn()

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  userApi: { listMyOrganizations: () => listMyOrganizations() as unknown },
  organizationApi: {
    createOrganization: (...args: unknown[]) => createOrganization(...args) as unknown,
    updateOrganization: (...args: unknown[]) => updateOrganization(...args) as unknown,
    deleteOrganization: (...args: unknown[]) => deleteOrganization(...args) as unknown,
  },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

describe('useOrganizationMutations', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('sends null address as null', async () => {
    updateOrganization.mockResolvedValue({ id: 'o1', name: 'Nord' })
    const { result } = renderHookWithClient(() => useOrganizationMutations())

    result.current.updateOrganization.mutate({
      orgId: 'o1',
      name: 'Nord',
      address: null,
      contactPersonId: null,
    })

    await waitFor(() => expect(updateOrganization).toHaveBeenCalledTimes(1))
    expect(updateOrganization).toHaveBeenCalledWith({
      orgId: 'o1',
      organizationUpdateRequest: { name: 'Nord', address: null, contactPersonId: null },
    })
  })

  it('does not toast on a 409 name conflict', async () => {
    updateOrganization.mockRejectedValue({ response: { status: 409 } })
    const { result } = renderHookWithClient(() => useOrganizationMutations())

    result.current.updateOrganization.mutate({
      orgId: 'o1',
      name: 'Nord',
      address: null,
      contactPersonId: null,
    })

    await waitFor(() => expect(updateOrganization).toHaveBeenCalledTimes(1))
    expect(showToast).not.toHaveBeenCalled()
  })

  it('does not toast on a 422 non-member contact person', async () => {
    updateOrganization.mockRejectedValue({ response: { status: 422 } })
    const { result } = renderHookWithClient(() => useOrganizationMutations())

    result.current.updateOrganization.mutate({
      orgId: 'o1',
      name: 'Nord',
      address: null,
      contactPersonId: 'u9',
    })

    await waitFor(() => expect(updateOrganization).toHaveBeenCalledTimes(1))
    expect(showToast).not.toHaveBeenCalled()
  })

  it('toasts on a delete conflict with an explanatory message', async () => {
    deleteOrganization.mockRejectedValue({ response: { status: 409 } })
    const { result } = renderHookWithClient(() => useOrganizationMutations())

    result.current.deleteOrganization.mutate({ orgId: 'o1' })

    await waitFor(() => expect(deleteOrganization).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith(expect.stringContaining('Unterorganisationen'), 'error')
  })

  // The caller navigates to the new organization (or away from a deleted one) in
  // onSuccess; the settings layout resolves it from the reachable list.
  it.each([
    ['create', { parentId: 'amt', name: 'West' }],
    ['delete', { orgId: 'west' }],
  ] as const)(
    'runs the caller onSuccess of %s only after the reachable list was refetched',
    async (kind, variables) => {
      const events: string[] = []
      createOrganization.mockResolvedValue({ id: 'west', name: 'West' })
      deleteOrganization.mockResolvedValue(undefined)
      listMyOrganizations.mockImplementation(async () => {
        await new Promise((resolve) => setTimeout(resolve, 10))
        events.push('reachable refetched')
        return []
      })
      const { result } = renderHookWithClient(() => {
        useQuery(userQueries.myOrganizations())
        return useOrganizationMutations()
      })
      await waitFor(() => expect(events).toEqual(['reachable refetched']))

      const onSuccess = () => events.push('caller onSuccess')
      if (kind === 'create') {
        result.current.createOrganization.mutate(variables, { onSuccess })
      } else {
        result.current.deleteOrganization.mutate(variables, { onSuccess })
      }

      await waitFor(() => expect(events).toContain('caller onSuccess'))
      expect(events).toEqual(['reachable refetched', 'reachable refetched', 'caller onSuccess'])
    },
  )
})
