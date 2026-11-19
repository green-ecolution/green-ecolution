import { describe, it, expect, vi, beforeEach } from 'vitest'
import { act } from 'react'
import { renderHook, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import type { IrrigationSettingsResponse } from '@/api/backendApi'
import { irrigationApi } from '../../api/client'
import { irrigationSettingsQueries } from '../../api/settings'
import { useIrrigationDraftState, useIrrigationSettingsDraft } from './useIrrigationSettingsDraft'

const getIrrigationSettings = vi.spyOn(irrigationApi, 'getIrrigationSettings')
const updateIrrigationSettings = vi.spyOn(irrigationApi, 'updateIrrigationSettings')

const response = (waterDemand = 120, ttlSecs = 86_400): IrrigationSettingsResponse => ({
  waterDemand: { value: waterDemand, inheritedValue: 80 },
  justWateredTtlSecs: { value: ttlSecs, inheritedValue: 86_400 },
})

describe('useIrrigationDraftState', () => {
  it('starts from the values in force and is clean', () => {
    const { result } = renderHook(() => useIrrigationDraftState())
    act(() => result.current.load(response()))
    expect(result.current.draft).toEqual({ waterDemand: '120', justWateredTtlHours: '24' })
    expect(result.current.dirty).toBe(false)
    expect(result.current.toRequest()).toBeNull()
  })

  it('sends only the field that moved', () => {
    const { result } = renderHook(() => useIrrigationDraftState())
    act(() => result.current.load(response()))
    act(() => result.current.setText('waterDemand', '90'))
    expect(result.current.toRequest()).toEqual({ waterDemand: 90 })
  })

  it('sends the usual value as a plain number, the backend lets it follow again', () => {
    const { result } = renderHook(() => useIrrigationDraftState())
    act(() => result.current.load(response()))
    act(() => result.current.setText('waterDemand', '80'))
    expect(result.current.toRequest()).toEqual({ waterDemand: 80 })
  })

  it('converts hours to seconds', () => {
    const { result } = renderHook(() => useIrrigationDraftState())
    act(() => result.current.load(response()))
    act(() => result.current.setText('justWateredTtlHours', '12'))
    expect(result.current.toRequest()).toEqual({ justWateredTtlSecs: 43_200 })
  })

  it('builds no request while a value is invalid', () => {
    const { result } = renderHook(() => useIrrigationDraftState())
    act(() => result.current.load(response()))
    act(() => result.current.setText('waterDemand', ''))
    expect(result.current.valid).toBe(false)
    expect(result.current.toRequest()).toBeNull()
  })

  it('reports a value outside the domain range', () => {
    const { result } = renderHook(() => useIrrigationDraftState())
    act(() => result.current.load(response()))
    act(() => result.current.setText('waterDemand', '5000'))
    expect(result.current.issues.waterDemand).not.toBeNull()
    expect(result.current.valid).toBe(false)
  })
})

const loaded = async () => {
  getIrrigationSettings.mockResolvedValue(response())
  const hook = renderHookWithClient(() => useIrrigationSettingsDraft('nord', { enabled: true }))
  await waitFor(() => expect(hook.result.current.loading).toBe(false))
  return hook
}

describe('useIrrigationSettingsDraft', () => {
  beforeEach(() => vi.clearAllMocks())

  it('loads the organization settings into a clean draft', async () => {
    const { result } = await loaded()

    expect(getIrrigationSettings).toHaveBeenCalledWith({ orgId: 'nord' })
    expect(result.current.dirty).toBe(false)
    expect(result.current.valid).toBe(true)
    expect(result.current.server).toEqual(response())
    expect(result.current.draft?.justWateredTtlHours).toBe('24')
  })

  it('saves only the field that moved and reloads its settings', async () => {
    updateIrrigationSettings.mockResolvedValue(response())
    const { result, queryClient } = await loaded()
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    act(() => result.current.setText('waterDemand', '95'))
    expect(result.current.dirty).toBe(true)
    await act(() => result.current.save())

    expect(updateIrrigationSettings).toHaveBeenCalledWith({
      orgId: 'nord',
      irrigationSettingsUpdateRequest: { waterDemand: 95 },
    })
    expect(invalidate).toHaveBeenCalledWith({
      queryKey: irrigationSettingsQueries.byOrganization('nord').queryKey,
    })
  })

  it('rejects a failed save so the page can report it', async () => {
    updateIrrigationSettings.mockRejectedValue({ response: { status: 500 } })
    const { result } = await loaded()

    act(() => result.current.setText('justWateredTtlHours', '48'))

    await act(async () => {
      await expect(result.current.save()).rejects.toBeDefined()
    })
    expect(result.current.draft?.justWateredTtlHours).toBe('48')
  })

  // `lastChange.changedAt` is a Date, so every refetch hands back a new object;
  // only a change in content may replace what was typed.
  it('keeps a typed value when the settings come back unchanged', async () => {
    const { result, queryClient } = await loaded()

    act(() => result.current.setText('justWateredTtlHours', '48'))
    act(() => {
      queryClient.setQueryData(
        irrigationSettingsQueries.byOrganization('nord').queryKey,
        response(),
      )
    })

    expect(result.current.draft?.justWateredTtlHours).toBe('48')
  })

  it('takes over a value that changed on the server', async () => {
    const { result, queryClient } = await loaded()

    act(() => result.current.setText('justWateredTtlHours', '48'))
    act(() => {
      queryClient.setQueryData(
        irrigationSettingsQueries.byOrganization('nord').queryKey,
        response(120, 259_200),
      )
    })

    await waitFor(() => expect(result.current.draft?.justWateredTtlHours).toBe('72'))
  })

  it('drops the edits on reset', async () => {
    const { result } = await loaded()

    act(() => result.current.setText('justWateredTtlHours', '48'))
    act(() => result.current.reset())

    expect(result.current.dirty).toBe(false)
    expect(result.current.draft?.justWateredTtlHours).toBe('24')
  })

  it('asks nothing while the module is off in the organization', () => {
    const { result } = renderHookWithClient(() =>
      useIrrigationSettingsDraft('nord', { enabled: false }),
    )

    expect(getIrrigationSettings).not.toHaveBeenCalled()
    expect(result.current.dirty).toBe(false)
  })
})
