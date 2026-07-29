import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'

const createRole = vi.fn()
const updateRole = vi.fn()
const deleteRole = vi.fn()

vi.mock('@/api/backendApi', () => ({
  roleApi: {
    createRole: (...args: unknown[]) => createRole(...args) as unknown,
    updateRole: (...args: unknown[]) => updateRole(...args) as unknown,
    deleteRole: (...args: unknown[]) => deleteRole(...args) as unknown,
  },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const { useRoleMutations } = await import('./useRoleMutations')

const renderMutations = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const invalidate = vi.spyOn(queryClient, 'invalidateQueries')
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return { ...renderHook(() => useRoleMutations(), { wrapper }), invalidate }
}

describe('useRoleMutations', () => {
  beforeEach(() => vi.clearAllMocks())

  it('creates a role with the full permission set', async () => {
    createRole.mockResolvedValue({ id: 'new-id' })
    const { result } = renderMutations()

    result.current.createRole.mutate({
      orgId: 'org-1',
      name: 'Bezirksleiter Nord',
      description: 'Kopie von Routenplanung',
      permissions: ['tree:read', 'watering_plan:read'],
    })

    await waitFor(() => expect(createRole).toHaveBeenCalled())
    expect(createRole).toHaveBeenCalledWith({
      orgId: 'org-1',
      roleCreateRequest: {
        name: 'Bezirksleiter Nord',
        description: 'Kopie von Routenplanung',
        permissions: ['tree:read', 'watering_plan:read'],
      },
    })
    expect(showToast).toHaveBeenCalledWith('Rolle angelegt')
  })

  it('invalidates the roles list and the current user after an update', async () => {
    updateRole.mockResolvedValue({ id: 'role-1' })
    const { result, invalidate } = renderMutations()

    result.current.updateRole.mutate({
      roleId: 'role-1',
      name: 'Praktikant',
      description: null,
      permissions: ['tree:read'],
    })

    await waitFor(() => expect(showToast).toHaveBeenCalledWith('Gespeichert'))
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['roles'] })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['users', 'me'] })
  })

  it('deletes a role', async () => {
    deleteRole.mockResolvedValue(undefined)
    const { result } = renderMutations()

    result.current.deleteRole.mutate({ roleId: 'role-1' })

    await waitFor(() => expect(deleteRole).toHaveBeenCalledWith({ roleId: 'role-1' }))
    expect(showToast).toHaveBeenCalledWith('Rolle gelöscht')
  })

  it('toasts an error for a non-409 create failure', async () => {
    createRole.mockRejectedValue({ response: { status: 403 } })
    const { result } = renderMutations()

    result.current.createRole.mutate({
      orgId: 'org-1',
      name: 'Bezirksleiter Nord',
      description: null,
      permissions: ['tree:delete'],
    })

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith('Die Rolle konnte nicht gespeichert werden.', 'error'),
    )
  })

  it('does not toast on a 409 create conflict', async () => {
    createRole.mockRejectedValue({ response: { status: 409 } })
    const { result } = renderMutations()

    result.current.createRole.mutate({
      orgId: 'org-1',
      name: 'Bezirksleiter Nord',
      description: null,
      permissions: ['tree:read'],
    })

    await waitFor(() => expect(createRole).toHaveBeenCalled())
    expect(showToast).not.toHaveBeenCalled()
  })

  it('toasts an error when deletion fails', async () => {
    deleteRole.mockRejectedValue({ response: { status: 500 } })
    const { result } = renderMutations()

    result.current.deleteRole.mutate({ roleId: 'role-1' })

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith('Die Rolle konnte nicht gelöscht werden.', 'error'),
    )
  })
})
