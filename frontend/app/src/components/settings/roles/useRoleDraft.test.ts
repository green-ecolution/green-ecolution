import { describe, it, expect } from 'vitest'
import { act, renderHook } from '@testing-library/react'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import type { Role } from '@/api/backendApi'
import { useRoleDraft } from './useRoleDraft'

const role = (overrides: Partial<Role> = {}): Role => ({
  id: 'r1',
  name: 'Praktikant',
  description: 'Eingeschränkt',
  isTemplate: false,
  permissions: ['tree:read'],
  ...overrides,
})

describe('useRoleDraft', () => {
  it('starts empty', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    expect(result.current.draft).toBeNull()
    expect(result.current.dirty).toBe(false)
  })

  it('loads an existing role without being dirty', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.editExisting(role()))
    expect(result.current.draft?.kind).toBe('existing')
    expect(result.current.draft?.name).toBe('Praktikant')
    expect(result.current.dirty).toBe(false)
  })

  it('becomes dirty on a name change and clean again on revert', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.editExisting(role()))
    act(() => result.current.setName('Praktikantin'))
    expect(result.current.dirty).toBe(true)
    act(() => result.current.setName('Praktikant'))
    expect(result.current.dirty).toBe(false)
  })

  it('becomes dirty on a permission change regardless of order', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.editExisting(role({ permissions: ['tree:read', 'sensor:read'] })))
    expect(result.current.dirty).toBe(false)
    act(() => result.current.toggle('tree:read'))
    expect(result.current.dirty).toBe(true)
  })

  it('applies a level preset to one resource only', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.editExisting(role({ permissions: ['sensor:read'] })))
    act(() => result.current.setLevel('tree', 'edit'))
    expect([...result.current.draft!.permissions].sort()).toEqual([
      'sensor:read',
      'tree:create',
      'tree:read',
      'tree:update',
    ])
  })

  it('is always dirty for a new draft', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.startNew())
    expect(result.current.draft?.kind).toBe('new')
    expect(result.current.draft?.name).toBe('')
    expect(result.current.dirty).toBe(true)
  })

  it('prefills a copy with the source name, description and permissions', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() =>
      result.current.startCopy(
        role({ name: 'Routenplanung', description: 'Planung', permissions: ['tree:read'] }),
      ),
    )
    expect(result.current.draft).toMatchObject({
      kind: 'new',
      name: 'Routenplanung (Kopie)',
      description: 'Kopie von Routenplanung',
    })
    expect([...result.current.draft!.permissions]).toEqual(['tree:read'])
    expect(result.current.draft!.clampedAway).toEqual([])
  })

  it('clamps a copy to the grantable set and records what it dropped', () => {
    const { result } = renderHook(() => useRoleDraft(new Set(['tree:read'])))
    act(() =>
      result.current.startCopy(
        role({ name: 'Administrator', permissions: ['tree:read', 'role:delete'] }),
      ),
    )
    expect([...result.current.draft!.permissions]).toEqual(['tree:read'])
    expect(result.current.draft!.clampedAway).toEqual(['role:delete'])
  })

  it('keeps unknown permissions through an edit', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.editExisting(role({ permissions: ['tree:read', 'report:export'] })))
    act(() => result.current.setLevel('tree', 'manage'))
    expect(result.current.draft!.permissions.has('report:export')).toBe(true)
  })

  it('discards the draft', () => {
    const { result } = renderHook(() => useRoleDraft(UNRESTRICTED))
    act(() => result.current.startNew())
    act(() => result.current.discard())
    expect(result.current.draft).toBeNull()
  })
})
