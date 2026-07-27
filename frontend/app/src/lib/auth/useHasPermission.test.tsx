import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { UNRESTRICTED, type Permissions } from './permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('./usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const { useHasPermission } = await import('./useHasPermission')

describe('useHasPermission', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  it('is true when the user holds one of the required permissions', () => {
    permissions.mockReturnValue(new Set(['tree:create']))
    const { result } = renderHook(() => useHasPermission(['tree:create']))
    expect(result.current).toBe(true)
  })

  it('is true when any one of several required permissions is held (OR)', () => {
    permissions.mockReturnValue(new Set(['tree_cluster:read']))
    const { result } = renderHook(() => useHasPermission(['tree:read', 'tree_cluster:read']))
    expect(result.current).toBe(true)
  })

  it('is false when none of the required permissions are held', () => {
    permissions.mockReturnValue(new Set(['tree:read']))
    const { result } = renderHook(() => useHasPermission(['tree:delete']))
    expect(result.current).toBe(false)
  })

  it('is true for an empty requirement', () => {
    const { result } = renderHook(() => useHasPermission([]))
    expect(result.current).toBe(true)
  })

  it('is true when access is unrestricted (auth bypass)', () => {
    permissions.mockReturnValue(UNRESTRICTED)
    const { result } = renderHook(() => useHasPermission(['vehicle:delete']))
    expect(result.current).toBe(true)
  })
})
