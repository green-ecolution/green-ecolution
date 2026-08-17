import { describe, it, expect } from 'vitest'
import { act } from 'react'
import { renderHook } from '@testing-library/react'
import { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import type { UserResponse } from '@/api/backendApi'
import { useMemberProfileDraft } from './useMemberProfileDraft'

const anna = {
  id: 'u-anna',
  firstName: 'Anna',
  lastName: 'Ahlmann',
  status: UserStatus.Available,
  drivingLicenses: [DrivingLicense.B],
  phoneNumber: '+49 461 1',
  employeeId: 'EMP-1',
} as UserResponse

describe('useMemberProfileDraft', () => {
  it('starts clean when loaded from a user', () => {
    const { result } = renderHook(() => useMemberProfileDraft())

    act(() => result.current.edit(anna))

    expect(result.current.draft?.userId).toBe('u-anna')
    expect(result.current.draft?.employeeId).toBe('EMP-1')
    expect(result.current.dirty).toBe(false)
  })

  it('becomes dirty on a real change', () => {
    const { result } = renderHook(() => useMemberProfileDraft())
    act(() => result.current.edit(anna))

    act(() => result.current.setPhoneNumber('+49 461 2'))

    expect(result.current.dirty).toBe(true)
  })

  it('is clean again when the value is typed back', () => {
    const { result } = renderHook(() => useMemberProfileDraft())
    act(() => result.current.edit(anna))

    act(() => result.current.setEmployeeId('EMP-2'))
    act(() => result.current.setEmployeeId('EMP-1'))

    expect(result.current.dirty).toBe(false)
  })

  it('ignores the order of driving licenses', () => {
    const { result } = renderHook(() => useMemberProfileDraft())
    act(() =>
      result.current.edit({
        ...anna,
        drivingLicenses: [DrivingLicense.B, DrivingLicense.Ce],
      }),
    )

    act(() => result.current.setDrivingLicenses([DrivingLicense.Ce, DrivingLicense.B]))

    expect(result.current.dirty).toBe(false)
  })

  it('re-bases on the server truth when the same user is loaded again', () => {
    const { result } = renderHook(() => useMemberProfileDraft())
    act(() => result.current.edit(anna))
    act(() => result.current.setPhoneNumber('+49 461 2'))

    act(() => result.current.edit({ ...anna, phoneNumber: '+49 461 3' }))

    expect(result.current.draft?.phoneNumber).toBe('+49 461 3')
    expect(result.current.dirty).toBe(false)
  })

  it('drops the draft on discard', () => {
    const { result } = renderHook(() => useMemberProfileDraft())
    act(() => result.current.edit(anna))

    act(() => result.current.discard())

    expect(result.current.draft).toBeNull()
    expect(result.current.dirty).toBe(false)
  })

  it('carries avatarUrl passively without marking dirty', () => {
    const { result } = renderHook(() => useMemberProfileDraft())
    const withAvatar = { ...anna, avatarUrl: 'https://example.com/avatar.png' }

    // Load user with avatar
    act(() => result.current.edit(withAvatar))

    expect(result.current.draft?.avatarUrl).toBe('https://example.com/avatar.png')
    expect(result.current.dirty).toBe(false)

    // Edit a form field
    act(() => result.current.setPhoneNumber('+49 461 2'))

    expect(result.current.draft?.avatarUrl).toBe('https://example.com/avatar.png')
    expect(result.current.dirty).toBe(true)

    // Type back to original value
    act(() => result.current.setPhoneNumber('+49 461 1'))

    // Only avatarUrl differs from baseline, but it never triggers dirty since it's not in canonical
    expect(result.current.draft?.avatarUrl).toBe('https://example.com/avatar.png')
    expect(result.current.dirty).toBe(false)
  })
})
