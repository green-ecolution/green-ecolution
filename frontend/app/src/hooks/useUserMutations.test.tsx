import { describe, it, expect, vi, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'

const assignUserRole = vi.fn()
const revokeUserRole = vi.fn()
const setUserOrganization = vi.fn()
const updateUser = vi.fn()
const createUser = vi.fn()
vi.mock('@/api/backendApi', () => ({
  userApi: { assignUserRole, revokeUserRole, setUserOrganization, updateUser, createUser },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const { useUserMutations } = await import('./useUserMutations')

const withClient =
  (client: QueryClient) =>
  ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )

const setup = () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const invalidate = vi.spyOn(client, 'invalidateQueries')
  const { result } = renderHook(() => useUserMutations(), { wrapper: withClient(client) })
  return { result, invalidate }
}

describe('useUserMutations', () => {
  beforeEach(() => vi.clearAllMocks())

  it('invalidates the user list after assigning a role', async () => {
    assignUserRole.mockResolvedValue({ id: 'r-1', name: 'Gärtner' })
    const { result, invalidate } = setup()

    result.current.assignRole.mutate({ userId: 'u-1', roleId: 'r-1' })

    await waitFor(() => expect(result.current.assignRole.isSuccess).toBe(true))
    expect(assignUserRole).toHaveBeenCalledWith({
      userId: 'u-1',
      assignRoleRequest: { roleId: 'r-1' },
    })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['users'] })
  })

  it('refreshes the list and the member counts after adding a person', async () => {
    createUser.mockResolvedValue({
      user: { id: 'u-2' },
      initialPassword: 'Abcde-fghjk-23456-mnpqr',
    })
    const { result, invalidate } = setup()

    result.current.createUser.mutate({
      organizationId: 'o-1',
      firstName: 'Max',
      lastName: 'Mustermann',
      email: 'max@example.com',
      username: 'mmustermann',
      roleIds: [],
    })

    await waitFor(() => expect(result.current.createUser.isSuccess).toBe(true))
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['users'] })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['organizations'] })
  })

  it('does not toast a 403, because it belongs at the role picker', async () => {
    assignUserRole.mockRejectedValue({ response: { status: 403 } })
    const { result } = setup()

    result.current.assignRole.mutate({ userId: 'u-1', roleId: 'r-1' })

    await waitFor(() => expect(result.current.assignRole.isError).toBe(true))
    expect(showToast).not.toHaveBeenCalled()
  })

  it('does not toast a 409, because it belongs at the affected card', async () => {
    setUserOrganization.mockRejectedValue({ response: { status: 409 } })
    const { result } = setup()

    result.current.setOrganization.mutate({ userId: 'u-1', organizationId: 'o-1' })

    await waitFor(() => expect(result.current.setOrganization.isError).toBe(true))
    expect(showToast).not.toHaveBeenCalled()
  })

  it('toasts an unexpected failure', async () => {
    updateUser.mockRejectedValue({ response: { status: 500 } })
    const { result } = setup()

    result.current.updateProfile.mutate({
      userId: 'u-1',
      employeeId: null,
      phoneNumber: null,
      avatarUrl: 'https://example.com/avatar.png',
      status: 'available',
      drivingLicenses: [],
      wateringPlanSelectable: true,
    })

    await waitFor(() => expect(result.current.updateProfile.isError).toBe(true))
    expect(showToast).toHaveBeenCalledWith(expect.any(String), 'error')
  })

  it('passes the current avatarUrl through, since updateUser replaces the whole profile', async () => {
    updateUser.mockResolvedValue({ id: 'u-1' })
    const { result } = setup()

    result.current.updateProfile.mutate({
      userId: 'u-1',
      employeeId: null,
      phoneNumber: null,
      avatarUrl: 'https://example.com/avatar.png',
      status: 'available',
      drivingLicenses: [],
      wateringPlanSelectable: true,
    })

    await waitFor(() => expect(result.current.updateProfile.isSuccess).toBe(true))
    expect(updateUser).toHaveBeenCalledWith({
      userId: 'u-1',
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      userUpdateRequest: expect.objectContaining({
        avatarUrl: 'https://example.com/avatar.png',
        wateringPlanSelectable: true,
      }),
    })
  })
})
