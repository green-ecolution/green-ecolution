import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import type { WasteSettingsResponse } from '@green-ecolution/backend-client'
import { wasteBinApi } from '../../api/client'
import { wasteSettingsQueries } from '../../api/settings'
import { useWasteSettingsDraft } from './useWasteSettingsDraft'

const getWasteSettings = vi.spyOn(wasteBinApi, 'getWasteSettings')
const updateWasteSettings = vi.spyOn(wasteBinApi, 'updateWasteSettings')

const response = (value = 80): WasteSettingsResponse => ({
  fullThreshold: { value, inheritedValue: 80 },
})

const loaded = async (settings: WasteSettingsResponse = response()) => {
  getWasteSettings.mockResolvedValue(settings)
  const hook = renderHookWithClient(() => useWasteSettingsDraft('nord', { enabled: true }))
  await waitFor(() => expect(hook.result.current.loading).toBe(false))
  return hook
}

describe('useWasteSettingsDraft', () => {
  beforeEach(() => vi.clearAllMocks())

  it('starts clean and does not fetch while the module is off', () => {
    const { result } = renderHookWithClient(() => useWasteSettingsDraft('nord', { enabled: false }))
    expect(result.current.loading).toBe(true)
    expect(getWasteSettings).not.toHaveBeenCalled()
  })

  it('loads the settings into a clean draft', async () => {
    const { result } = await loaded()

    expect(getWasteSettings).toHaveBeenCalledWith({ orgId: 'nord' })
    expect(result.current.dirty).toBe(false)
    expect(result.current.valid).toBe(true)
    expect(result.current.draft).toEqual({ text: '80' })
  })

  it('treats 0 as invalid and sends nothing', async () => {
    const { result } = await loaded()

    act(() => result.current.setText('0'))

    expect(result.current.issue).not.toBeNull()
    expect(result.current.valid).toBe(false)
    expect(result.current.dirty).toBe(false)
    await act(() => result.current.save())
    expect(updateWasteSettings).not.toHaveBeenCalled()
  })

  it('rejects a fractional threshold as no whole number instead of rounding it', async () => {
    const { result } = await loaded()

    act(() => result.current.setText('80.5'))

    expect(result.current.issue?.key).toBe('settings.full_threshold.invalidFormat')
    expect(result.current.valid).toBe(false)
  })

  it('saves 70 through the settings endpoint and reloads', async () => {
    updateWasteSettings.mockResolvedValue(response(70))
    const { result, queryClient } = await loaded()
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    act(() => result.current.setText('70'))
    expect(result.current.dirty).toBe(true)
    await act(() => result.current.save())

    expect(updateWasteSettings).toHaveBeenCalledWith({
      orgId: 'nord',
      wasteSettingsUpdateRequest: { fullThreshold: 70 },
    })
    expect(invalidate).toHaveBeenCalledWith({
      queryKey: wasteSettingsQueries.byOrganization('nord').queryKey,
    })
  })

  it('sends the usual value as a plain number, the backend lets it follow again', async () => {
    updateWasteSettings.mockResolvedValue(response())
    const { result } = await loaded(response(70))

    act(() => result.current.setText('80'))
    await act(() => result.current.save())

    expect(updateWasteSettings).toHaveBeenCalledWith({
      orgId: 'nord',
      wasteSettingsUpdateRequest: { fullThreshold: 80 },
    })
  })

  it('builds no request while the text is not numeric', async () => {
    const { result } = await loaded(response(70))
    act(() => result.current.setText('abc'))
    expect(result.current.valid).toBe(false)
    expect(result.current.dirty).toBe(false)
  })

  it('rejects a failed save and keeps the typed value', async () => {
    updateWasteSettings.mockRejectedValue({ response: { status: 500 } })
    const { result } = await loaded(response(70))

    act(() => result.current.setText('60'))
    await act(async () => {
      await expect(result.current.save()).rejects.toBeDefined()
    })
    expect(result.current.draft?.text).toBe('60')
  })

  it('keeps a typed value when the settings come back unchanged', async () => {
    const { result, queryClient } = await loaded(response(70))

    act(() => result.current.setText('60'))
    act(() => {
      queryClient.setQueryData(wasteSettingsQueries.byOrganization('nord').queryKey, response(70))
    })

    expect(result.current.draft?.text).toBe('60')
  })

  it('resets the draft to what the server holds', async () => {
    const { result } = await loaded(response(70))
    act(() => result.current.setText('60'))
    act(() => result.current.reset())
    expect(result.current.draft?.text).toBe('70')
  })
})
