import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import type { WasteSettingsResponse } from '@green-ecolution/backend-client'
import { wasteBinApi } from '../../api/client'
import { wasteSettingsQueries } from '../../api/settings'
import { useWasteSettingsDraft } from './useWasteSettingsDraft'

const getWasteSettings = vi.spyOn(wasteBinApi, 'getWasteSettings')
const updateWasteSettings = vi.spyOn(wasteBinApi, 'updateWasteSettings')

const inherited = {
  fullThreshold: { value: 80, origin: 'inherited', source: { id: 'parent' }, lastChange: null },
  enforcedBy: null,
} as unknown as WasteSettingsResponse

const own = {
  fullThreshold: { value: 70, origin: 'own', ownValue: 70, lastChange: null },
  enforcedBy: null,
} as unknown as WasteSettingsResponse

const loaded = async (response: WasteSettingsResponse = inherited) => {
  getWasteSettings.mockResolvedValue(response)
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
    expect(result.current.draft).toEqual({ own: false, text: '80' })
  })

  it('treats 0 as invalid and sends nothing', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn(true))
    act(() => result.current.setText('0'))

    expect(result.current.issue).not.toBeNull()
    expect(result.current.valid).toBe(false)
    expect(result.current.dirty).toBe(false)
    await act(() => result.current.save())
    expect(updateWasteSettings).not.toHaveBeenCalled()
  })

  it('rejects a fractional threshold as no whole number instead of rounding it', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn(true))
    act(() => result.current.setText('80.5'))

    expect(result.current.issue?.key).toBe('settings.full_threshold.invalidFormat')
    expect(result.current.valid).toBe(false)
  })

  it('saves 70 through the settings endpoint and reloads', async () => {
    updateWasteSettings.mockResolvedValue(own)
    const { result, queryClient } = await loaded()
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    act(() => result.current.setOwn(true))
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

  it('gives an own value back to inheritance with null', async () => {
    updateWasteSettings.mockResolvedValue(inherited)
    const { result } = await loaded(own)

    act(() => result.current.setOwn(false))
    await act(() => result.current.save())

    expect(updateWasteSettings).toHaveBeenCalledWith({
      orgId: 'nord',
      wasteSettingsUpdateRequest: { fullThreshold: null },
    })
  })

  it('takes over the inherited value as the starting point', async () => {
    const { result } = await loaded()
    act(() => result.current.setOwn(true))
    expect(result.current.draft?.text).toBe('80')
  })

  it('does not silently give up an own value when the text is not numeric', async () => {
    const { result } = await loaded(own)
    act(() => result.current.setText('abc'))
    expect(result.current.valid).toBe(false)
    expect(result.current.dirty).toBe(false)
  })

  it('rejects a failed save and keeps the typed value', async () => {
    updateWasteSettings.mockRejectedValue({ response: { status: 500 } })
    const { result } = await loaded(own)

    act(() => result.current.setText('60'))
    await act(async () => {
      await expect(result.current.save()).rejects.toBeDefined()
    })
    expect(result.current.draft?.text).toBe('60')
  })

  it('keeps a typed value when the settings come back unchanged', async () => {
    const { result, queryClient } = await loaded(own)

    act(() => result.current.setText('60'))
    act(() => {
      queryClient.setQueryData(wasteSettingsQueries.byOrganization('nord').queryKey, { ...own })
    })

    expect(result.current.draft?.text).toBe('60')
  })

  it('resets the draft to what the server holds', async () => {
    const { result } = await loaded(own)
    act(() => result.current.setText('60'))
    act(() => result.current.reset())
    expect(result.current.draft?.text).toBe('70')
  })
})
