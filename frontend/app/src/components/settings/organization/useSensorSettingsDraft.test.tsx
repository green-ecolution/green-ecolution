import { describe, it, expect, beforeEach, vi } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import { settingsApi, type OrganizationSettingsResponse } from '@/api/backendApi'
import { useSensorSettingsDraft } from './useSensorSettingsDraft'

const getOrganizationSettings = vi.spyOn(settingsApi, 'getOrganizationSettings')
const updateSettings = vi.spyOn(settingsApi, 'updateOrganizationSettings')

const settings = {
  sensorOfflineAfterSecs: { value: 172_800, inheritedValue: 86_400 },
  defectStreak: { value: 3, inheritedValue: 3 },
  mapView: { value: { center: [54.79, 9.43] }, inheritedValue: { center: [54.79, 9.43] } },
} as OrganizationSettingsResponse

const renderDraft = ({ enabled = true } = {}) =>
  renderHookWithClient(() => useSensorSettingsDraft('nord', { enabled }))

describe('useSensorSettingsDraft', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    getOrganizationSettings.mockResolvedValue(settings)
    updateSettings.mockResolvedValue(settings)
  })

  it('shows the offline time in hours and the streak as a count', async () => {
    const { result } = renderDraft()
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.draft).toEqual({ offlineAfterHours: '48', defectStreak: '3' })
    expect(result.current.dirty).toBe(false)
  })

  it('saves only the moved field, the offline time in seconds', async () => {
    const { result } = renderDraft()
    await waitFor(() => expect(result.current.loading).toBe(false))
    act(() => result.current.setText('offlineAfterHours', '24'))
    await act(() => result.current.save())
    expect(updateSettings).toHaveBeenCalledWith({
      orgId: 'nord',
      organizationSettingsUpdateRequest: { sensorOfflineAfterSecs: 86_400 },
    })
  })

  it('is invalid and not dirty while a count is a fraction', async () => {
    const { result } = renderDraft()
    await waitFor(() => expect(result.current.loading).toBe(false))
    act(() => result.current.setText('defectStreak', '2.5'))
    expect(result.current.valid).toBe(false)
    expect(result.current.dirty).toBe(false)
  })

  it('does not fetch while disabled', () => {
    renderDraft({ enabled: false })
    expect(getOrganizationSettings).not.toHaveBeenCalled()
  })
})
