import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { useQuery } from '@tanstack/react-query'
import { organizationQueries, userQueries } from '@/api/queries'
import { render, renderHookWithClient, screen, userEvent } from '@/test/utils'
import { useOrganizationMutations } from './useOrganizationMutations'

const createOrganization = vi.fn()
const updateOrganization = vi.fn()
const deleteOrganization = vi.fn()
const listMyOrganizations = vi.fn()
const listOrganizations = vi.fn()

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  userApi: { listMyOrganizations: () => listMyOrganizations() as unknown },
  organizationApi: {
    listOrganizations: (...args: unknown[]) => listOrganizations(...args) as unknown,
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

  // The caller navigates to the new organization in onSuccess; the settings
  // layout resolves it from the reachable list, which must know it by then.
  it('runs the caller onSuccess of create only after the reachable list was refetched', async () => {
    const events: string[] = []
    createOrganization.mockResolvedValue({ id: 'west', name: 'West' })
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

    result.current.createOrganization.mutate(
      { parentId: 'amt', name: 'West' },
      { onSuccess: () => events.push('caller onSuccess') },
    )

    await waitFor(() => expect(events).toContain('caller onSuccess'))
    expect(events).toEqual(['reachable refetched', 'reachable refetched', 'caller onSuccess'])
  })

  // The refetched list no longer holds the deleted organization, which turns its
  // still open page into "Kein Zugriff"; the caller must navigate away first.
  it('runs the caller onSuccess of delete before the reachable refetch, on a stale-marked list', async () => {
    const events: string[] = []
    deleteOrganization.mockResolvedValue(undefined)
    listMyOrganizations.mockImplementation(async () => {
      await new Promise((resolve) => setTimeout(resolve, 10))
      events.push('reachable refetched')
      return []
    })
    const { result, queryClient } = renderHookWithClient(() => {
      useQuery(userQueries.myOrganizations())
      return useOrganizationMutations()
    })
    await waitFor(() => expect(events).toEqual(['reachable refetched']))

    let invalidatedOnSuccess: boolean | undefined
    result.current.deleteOrganization.mutate(
      { orgId: 'west' },
      {
        onSuccess: () => {
          events.push('caller onSuccess')
          invalidatedOnSuccess = queryClient
            .getQueryCache()
            .find({ queryKey: userQueries.myOrganizations().queryKey })?.state.isInvalidated
        },
      },
    )

    await waitFor(() => expect(events).toHaveLength(3))
    expect(events).toEqual(['reachable refetched', 'caller onSuccess', 'reachable refetched'])
    expect(invalidatedOnSuccess).toBe(true)
  })

  it('navigates away from a deleted organization even though its page loses access', async () => {
    deleteOrganization.mockResolvedValue(undefined)
    listMyOrganizations
      .mockResolvedValueOnce([{ id: 'west', name: 'West', parentId: 'amt' }])
      .mockResolvedValue([])
    // Another invalidated query that takes longer, as GET /organizations or /users/me
    // do in the browser: the reachable list lands first and the page loses access.
    listOrganizations.mockImplementation(async () => {
      await new Promise((resolve) => setTimeout(resolve, 50))
      return []
    })
    const navigate = vi.fn()

    const Page = () => {
      const { deleteOrganization: remove } = useOrganizationMutations()
      return (
        <button
          type="button"
          onClick={() => remove.mutate({ orgId: 'west' }, { onSuccess: () => void navigate() })}
        >
          Löschen
        </button>
      )
    }
    // Stands in for SettingsOrganizationOutlet, which shows "Kein Zugriff" in place
    // of the page once the reachable list lacks the organization.
    const Outlet = () => {
      useQuery(organizationQueries.list())
      const { data } = useQuery(userQueries.myOrganizations())
      if (!data) return null
      return data.some((org) => org.id === 'west') ? <Page /> : <p>Kein Zugriff</p>
    }

    render(<Outlet />)
    await userEvent.click(await screen.findByRole('button', { name: 'Löschen' }))

    expect(await screen.findByText('Kein Zugriff')).toBeInTheDocument()
    await new Promise((resolve) => setTimeout(resolve, 100))
    expect(navigate).toHaveBeenCalledTimes(1)
  })
})
