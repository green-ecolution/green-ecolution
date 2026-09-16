import { describe, it, expect } from 'vitest'
import { act } from 'react'
import { renderHook } from '@testing-library/react'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import { useOrganizationSettingsDraft } from './useOrganizationSettingsDraft'

const response = {
  waterDemand: { value: 80, origin: 'inherited', source: { id: 'parent' }, lastChange: null },
  justWateredTtlSecs: { value: 86400, origin: 'own', ownValue: 86400, lastChange: null },
  descendantsMayOverride: true,
  enforcedBy: null,
} as OrganizationSettingsResponse

describe('useOrganizationSettingsDraft', () => {
  it('starts clean and sends nothing', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    expect(result.current.dirty).toBe(false)
    expect(result.current.toRequest()).toBeNull()
  })

  it('sends null when an own value is given back to inheritance', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setOwn('justWateredTtlHours', false))
    expect(result.current.toRequest()).toEqual({ justWateredTtlSecs: null })
  })

  it('converts hours back into seconds', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setText('justWateredTtlHours', '12'))
    expect(result.current.toRequest()).toEqual({ justWateredTtlSecs: 43200 })
  })

  it('takes over an inherited value as the starting point when own is switched on', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setOwn('waterDemand', true))
    expect(result.current.draft?.waterDemand.text).toBe('80')
    expect(result.current.toRequest()).toEqual({ waterDemand: 80 })
  })

  it('reports a value outside the domain range and refuses to submit', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setOwn('waterDemand', true))
    act(() => result.current.setText('waterDemand', '5000'))
    expect(result.current.errors.waterDemand).not.toBeNull()
    expect(result.current.valid).toBe(false)
  })

  it('does not silently give up an own value when the text is not numeric', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setOwn('waterDemand', true))
    act(() => result.current.setText('waterDemand', 'abc'))
    expect(result.current.valid).toBe(false)
    // Number('abc') is NaN, which JSON.stringify turns into null — that would
    // read server-side as "give up the own value". Must not happen.
    expect(result.current.toRequest()).toBeNull()
  })
})
