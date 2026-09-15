import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import { useSettingsMutations } from './useSettingsMutations'

const updateOrganizationSettings = vi.fn()

vi.mock('@/api/backendApi', () => ({
  settingsApi: {
    updateOrganizationSettings: (...args: unknown[]) =>
      updateOrganizationSettings(...args) as unknown,
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

    result.current.updateSettings.mutate({ orgId: 'org-1', body: { waterDemand: 120 } })

    await waitFor(() => expect(updateOrganizationSettings).toHaveBeenCalledTimes(1))
    expect(updateOrganizationSettings).toHaveBeenCalledWith({
      orgId: 'org-1',
      organizationSettingsUpdateRequest: { waterDemand: 120 },
    })
    await waitFor(() => expect(showToast).toHaveBeenCalledWith('Einstellungen gespeichert'))
  })

  it('reports the ancestor lock instead of a generic error', async () => {
    updateOrganizationSettings.mockRejectedValue({ response: { status: 409 } })
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({ orgId: 'org-1', body: { waterDemand: 120 } })

    await waitFor(() => expect(updateOrganizationSettings).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith(expect.stringContaining('gesperrt'), 'error')
  })

  it('reports a generic error for anything else', async () => {
    updateOrganizationSettings.mockRejectedValue({ response: { status: 500 } })
    const { result } = renderHookWithClient(() => useSettingsMutations())

    result.current.updateSettings.mutate({ orgId: 'org-1', body: { waterDemand: 120 } })

    await waitFor(() => expect(updateOrganizationSettings).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith(expect.not.stringContaining('gesperrt'), 'error')
  })
})
