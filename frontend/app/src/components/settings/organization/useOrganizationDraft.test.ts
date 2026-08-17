import { describe, it, expect } from 'vitest'
import { act, renderHook } from '@testing-library/react'
import type { OrganizationDetailResponse } from '@/api/backendApi'
import { useOrganizationDraft } from './useOrganizationDraft'

const detail = (
  overrides: Partial<OrganizationDetailResponse> = {},
): OrganizationDetailResponse => ({
  id: 'nord',
  parentId: 'amt',
  name: 'Stadtgärtnerei Nord',
  address: { street: 'Nordergraben 12', postalCode: '24937', city: 'Flensburg' },
  contactPerson: null,
  memberCount: 4,
  createdAt: null,
  ...overrides,
})

describe('useOrganizationDraft', () => {
  it('starts clean after edit', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail()))
    expect(result.current.dirty).toBe(false)
    expect(result.current.draft?.postalCode).toBe('24937')
  })

  it('becomes dirty on a name change and clean again on revert', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail()))
    act(() => result.current.setName('Stadtgärtnerei Süd'))
    expect(result.current.dirty).toBe(true)
    act(() => result.current.setName('Stadtgärtnerei Nord'))
    expect(result.current.dirty).toBe(false)
  })

  it('treats an empty address as complete', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail({ address: null })))
    expect(result.current.addressComplete).toBe(true)
    expect(result.current.addressErrors).toEqual({})
  })

  it('flags the missing parts of a partial address', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail({ address: null })))
    act(() => result.current.setCity('Flensburg'))
    expect(result.current.addressComplete).toBe(false)
    expect(result.current.addressErrors.street).toBe('Straße fehlt')
    expect(result.current.addressErrors.postalCode).toBe('PLZ fehlt')
    expect(result.current.addressErrors.city).toBeUndefined()
  })

  it('ignores whitespace-only values when judging completeness', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail({ address: null })))
    act(() => result.current.setStreet('   '))
    expect(result.current.addressComplete).toBe(true)
  })

  it('treats a fully filled address as complete', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail({ address: null })))
    act(() => result.current.setStreet('Nordergraben 12'))
    act(() => result.current.setPostalCode('24937'))
    act(() => result.current.setCity('Flensburg'))
    expect(result.current.addressComplete).toBe(true)
    expect(result.current.addressErrors).toEqual({})
  })

  it('becomes dirty on a contact person change and clean again on revert', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail({ contactPerson: null })))
    act(() => result.current.setContactPersonId('user-1'))
    expect(result.current.dirty).toBe(true)
    act(() => result.current.setContactPersonId(null))
    expect(result.current.dirty).toBe(false)
  })

  it('baselines the contact person from the raw id, not the resolved person', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    // contactPerson absent means the identity provider could not resolve the id.
    act(() => result.current.edit(detail({ contactPersonId: 'user-1', contactPerson: null })))
    expect(result.current.draft?.contactPersonId).toBe('user-1')
    expect(result.current.dirty).toBe(false)
  })

  it('discard clears the draft', () => {
    const { result } = renderHook(() => useOrganizationDraft())
    act(() => result.current.edit(detail()))
    act(() => result.current.discard())
    expect(result.current.draft).toBeNull()
  })
})
