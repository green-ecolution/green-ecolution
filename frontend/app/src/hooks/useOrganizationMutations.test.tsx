import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import { useOrganizationMutations } from './useOrganizationMutations'

const createOrganization = vi.fn()
const updateOrganization = vi.fn()
const deleteOrganization = vi.fn()

vi.mock('@/api/backendApi', () => ({
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
})
