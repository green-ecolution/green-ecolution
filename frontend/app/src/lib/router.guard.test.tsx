import { describe, it, expect, vi, beforeEach } from 'vitest'
import { isValidElement } from 'react'
import { QueryClient } from '@tanstack/react-query'
import type { UserResponse } from '@green-ecolution/backend-client'

const readAuthBypass = vi.fn(() => false)

vi.mock('@/lib/auth/runtimeConfig', () => ({
  readAuthBypass: () => readAuthBypass(),
}))

const { ForbiddenError, forbiddenErrorComponent, guardedRoute, requirePermission } =
  await import('./router')
const { default: Forbidden } = await import('@/components/layout/Forbidden')

const me = (permissions: string[]): UserResponse =>
  ({
    roles: [
      {
        id: 'role-1',
        name: 'Rolle',
        description: '',
        organizationId: null,
        permissions,
        createdAt: '2026-07-27T00:00:00Z',
      },
    ],
  }) as unknown as UserResponse

// Seeding the cache makes ensureQueryData resolve without hitting the queryFn.
const contextWith = (permissions: string[]) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  queryClient.setQueryData(['users', 'me'], me(permissions))
  return { context: { queryClient } }
}

describe('requirePermission', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    readAuthBypass.mockReturnValue(false)
  })

  it('passes when the user holds the required permission', async () => {
    await expect(
      requirePermission(['tree:read'])(contextWith(['tree:read'])),
    ).resolves.toBeUndefined()
  })

  it('throws ForbiddenError when the permission is missing', async () => {
    await expect(requirePermission(['tree:read'])(contextWith(['vehicle:read']))).rejects.toThrow(
      ForbiddenError,
    )
  })

  it('accepts any of an OR requirement', async () => {
    await expect(
      requirePermission(['tree:read', 'tree_cluster:read'])(contextWith(['tree_cluster:read'])),
    ).resolves.toBeUndefined()
  })

  it('passes without consulting the cache when auth is bypassed', async () => {
    readAuthBypass.mockReturnValue(true)
    const queryClient = new QueryClient()
    const spy = vi.spyOn(queryClient, 'ensureQueryData')

    await expect(
      requirePermission(['organization:delete'])({ context: { queryClient } }),
    ).resolves.toBeUndefined()
    expect(spy).not.toHaveBeenCalled()
  })
})

describe('forbiddenErrorComponent', () => {
  it('renders the Forbidden page for a ForbiddenError', () => {
    const element = forbiddenErrorComponent()({
      error: new ForbiddenError(),
      reset: () => undefined,
    })

    expect(isValidElement(element) && element.type).toBe(Forbidden)
  })

  it('delegates other errors to the provided fallback', () => {
    const fallback = vi.fn(() => <div data-testid="original" />)

    const element = forbiddenErrorComponent(fallback)({
      error: new Error('boom'),
      reset: () => undefined,
    })

    expect(fallback).toHaveBeenCalledOnce()
    // Asserting on `type` keeps the check free of untyped props access.
    expect(isValidElement(element) && element.type).toBe('div')
  })

  it('still renders Forbidden when a fallback exists', () => {
    const fallback = vi.fn(() => <div data-testid="original" />)

    const element = forbiddenErrorComponent(fallback)({
      error: new ForbiddenError(),
      reset: () => undefined,
    })

    expect(fallback).not.toHaveBeenCalled()
    expect(isValidElement(element) && element.type).toBe(Forbidden)
  })
})

describe('guardedRoute', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    readAuthBypass.mockReturnValue(false)
  })

  it('keeps the wrapped options and adds the guard', async () => {
    const component = () => null
    const options = guardedRoute(['tree:read'], { component })

    expect(options.component).toBe(component)
    await expect(options.beforeLoad(contextWith(['vehicle:read']))).rejects.toThrow(ForbiddenError)
  })

  it('composes an existing errorComponent', () => {
    const fallback = vi.fn(() => <span data-testid="entity-not-found" />)
    const options = guardedRoute(['tree:read'], { errorComponent: fallback })

    const element = options.errorComponent({ error: new Error('boom'), reset: () => undefined })

    expect(fallback).toHaveBeenCalledOnce()
    expect(isValidElement(element) && element.type).toBe('span')
  })
})
