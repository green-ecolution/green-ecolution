import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import { useSettingsMutations } from './useSettingsMutations'

const updateOrganizationSettings = vi.fn()
const updateIrrigationSettings = vi.fn()

vi.mock('@/api/backendApi', () => ({
  settingsApi: {
    updateOrganizationSettings: (...args: unknown[]) =>
      updateOrganizationSettings(...args) as unknown,
  },
}))

vi.mock('../../../modules/irrigation/src/api/client', () => ({
  irrigationApi: {
    updateIrrigationSettings: (...args: unknown[]) => updateIrrigationSettings(...args) as unknown,
  },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

describe('useSettingsMutations', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('sends the update request for the given organization', async () => {
    updateOrganizationSettings.mockResolvedValue({})
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({
      orgId: 'org-1',
      body: { descendantsMayOverride: false },
    })

    await waitFor(() => expect(updateOrganizationSettings).toHaveBeenCalledTimes(1))
    expect(updateOrganizationSettings).toHaveBeenCalledWith({
      orgId: 'org-1',
      organizationSettingsUpdateRequest: { descendantsMayOverride: false },
    })
    await waitFor(() => expect(showToast).toHaveBeenCalledWith('Einstellungen gespeichert'))
  })

  it('reports the ancestor lock instead of a generic error', async () => {
    updateOrganizationSettings.mockRejectedValue({ response: { status: 409 } })
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({
      orgId: 'org-1',
      body: { descendantsMayOverride: false },
    })

    await waitFor(() => expect(updateOrganizationSettings).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith(expect.stringContaining('gesperrt'), 'error')
  })

  it('reports a generic error for anything else', async () => {
    updateOrganizationSettings.mockRejectedValue({ response: { status: 500 } })
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({
      orgId: 'org-1',
      body: { descendantsMayOverride: false },
    })

    await waitFor(() => expect(updateOrganizationSettings).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith(expect.not.stringContaining('gesperrt'), 'error')
  })

  it('sends the irrigation part to its own endpoint and skips the core one', async () => {
    updateIrrigationSettings.mockResolvedValue({})
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({ orgId: 'org-1', irrigationBody: { waterDemand: 120 } })

    await waitFor(() => expect(updateIrrigationSettings).toHaveBeenCalledTimes(1))
    expect(updateIrrigationSettings).toHaveBeenCalledWith({
      orgId: 'org-1',
      irrigationSettingsUpdateRequest: { waterDemand: 120 },
    })
    expect(updateOrganizationSettings).not.toHaveBeenCalled()
    await waitFor(() => expect(showToast).toHaveBeenCalledWith('Einstellungen gespeichert'))
  })

  it('reports success once, and only after both requests went through', async () => {
    updateOrganizationSettings.mockResolvedValue({})
    updateIrrigationSettings.mockResolvedValue({})
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({
      orgId: 'org-1',
      body: { descendantsMayOverride: false },
      irrigationBody: { waterDemand: 120 },
    })

    await waitFor(() => expect(showToast).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith('Einstellungen gespeichert')
  })

  it('reports an error when only one of the two requests fails', async () => {
    updateOrganizationSettings.mockResolvedValue({})
    updateIrrigationSettings.mockRejectedValue({ response: { status: 500 } })
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({
      orgId: 'org-1',
      body: { descendantsMayOverride: false },
      irrigationBody: { waterDemand: 120 },
    })

    await waitFor(() => expect(showToast).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith(expect.any(String), 'error')
  })
})
