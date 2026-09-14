import { describe, expect, it } from 'vitest'
import { act, renderHook } from '@testing-library/react'
import { usePluginPermissionDraft } from './usePluginPermissionDraft'

describe('usePluginPermissionDraft', () => {
  it('mirrors a plugin permission into the access set while untouched', () => {
    const { result } = renderHook(() => usePluginPermissionDraft())

    act(() => result.current.togglePermission('tree:create'))

    expect(result.current.permissions.has('tree:create')).toBe(true)
    expect(result.current.accessPermissions.has('tree:create')).toBe(true)
    expect(result.current.accessTouched).toBe(false)
  })

  it('touching the access set stops mirroring from then on', () => {
    const { result } = renderHook(() => usePluginPermissionDraft())

    act(() => result.current.toggleAccessPermission('tree:read'))
    expect(result.current.accessTouched).toBe(true)

    act(() => result.current.togglePermission('tree:create'))

    expect(result.current.permissions.has('tree:create')).toBe(true)
    expect(result.current.accessPermissions.has('tree:create')).toBe(false)
    expect(result.current.accessPermissions.has('tree:read')).toBe(true)
  })

  it('never resets accessTouched on further plugin-permission changes', () => {
    const { result } = renderHook(() => usePluginPermissionDraft())

    act(() => result.current.toggleAccessPermission('tree:read'))
    expect(result.current.accessTouched).toBe(true)

    act(() => result.current.togglePermission('tree:create'))
    act(() => result.current.togglePermission('tree:update'))
    act(() => result.current.togglePermission('tree:create')) // toggled back off

    expect(result.current.accessTouched).toBe(true)
  })

  it('reset returns both sets and the touched flag to their initial state', () => {
    const { result } = renderHook(() => usePluginPermissionDraft(['tree:create'], ['tree:create']))

    act(() => result.current.toggleAccessPermission('tree:read'))
    expect(result.current.accessTouched).toBe(true)
    expect(result.current.accessPermissions.has('tree:read')).toBe(true)

    act(() => result.current.reset(['tree:create'], ['tree:create']))

    expect([...result.current.permissions]).toEqual(['tree:create'])
    expect([...result.current.accessPermissions]).toEqual(['tree:create'])
    expect(result.current.accessTouched).toBe(false)
  })
})
