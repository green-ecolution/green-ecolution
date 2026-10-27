import { describe, it, expect, vi, beforeEach } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import {
  useSettingsMutations,
  useSettingsSave,
  useUpdateOrganizationModules,
} from './useSettingsMutations'

const updateOrganizationSettings = vi.fn()
const updateOrganizationModules = vi.fn()

vi.mock('@/api/backendApi', async () => ({
  ...(await vi.importActual<typeof import('@/api/backendApi')>('@/api/backendApi')),
  settingsApi: {
    updateOrganizationSettings: (...args: unknown[]) =>
      updateOrganizationSettings(...args) as unknown,
  },
  moduleApi: {
    updateOrganizationModules: (...args: unknown[]) =>
      updateOrganizationModules(...args) as unknown,
  },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

describe('useSettingsMutations', () => {
  beforeEach(() => vi.clearAllMocks())

  it('sends the update request for the given organization', async () => {
    updateOrganizationSettings.mockResolvedValue({})
    const { result, queryClient } = renderHookWithClient(() => useSettingsMutations())
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    await act(() =>
      result.current.updateSettings.mutateAsync({
        orgId: 'org-1',
        body: { descendantsMayOverride: false },
      }),
    )

    expect(updateOrganizationSettings).toHaveBeenCalledWith({
      orgId: 'org-1',
      organizationSettingsUpdateRequest: { descendantsMayOverride: false },
    })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['organizations', 'org-1', 'settings'] })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['users', 'me', 'map-view'] })
    // The page reports one outcome for every part it saved.
    expect(showToast).not.toHaveBeenCalled()
  })
})

describe('useUpdateOrganizationModules', () => {
  beforeEach(() => vi.clearAllMocks())

  it('sends the switches and reloads the active modules', async () => {
    updateOrganizationModules.mockResolvedValue({ modules: [] })
    const { result, queryClient } = renderHookWithClient(() => useUpdateOrganizationModules())
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    await act(() => result.current.mutateAsync({ orgId: 'org-1', modules: { irrigation: false } }))

    expect(updateOrganizationModules).toHaveBeenCalledWith({
      orgId: 'org-1',
      organizationModulesUpdateRequest: { modules: { irrigation: false } },
    })
    // Navigation and the module route guards read the `modules` root.
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['modules'] })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['organizations', 'org-1', 'modules'] })
  })

  it('reloads after a failure too, since another tab may have moved the switches', async () => {
    updateOrganizationModules.mockRejectedValue({ response: { status: 409 } })
    const { result, queryClient } = renderHookWithClient(() => useUpdateOrganizationModules())
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    await act(async () => {
      await expect(
        result.current.mutateAsync({ orgId: 'org-1', modules: { irrigation: false } }),
      ).rejects.toBeDefined()
    })

    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['organizations', 'org-1', 'modules'] })
  })
})

describe('useSettingsSave', () => {
  beforeEach(() => vi.clearAllMocks())

  it('reports success once, after every part went through', async () => {
    const { result } = renderHookWithClient(() => useSettingsSave())
    const first = vi.fn().mockResolvedValue(undefined)
    const second = vi.fn().mockResolvedValue(undefined)

    await act(() => result.current.save([[first, second]]))

    expect(first).toHaveBeenCalledTimes(1)
    expect(second).toHaveBeenCalledTimes(1)
    expect(showToast).toHaveBeenCalledTimes(1)
    expect(showToast).toHaveBeenCalledWith('Einstellungen gespeichert')
  })

  it('still sends the other parts when one fails, and reports the error', async () => {
    const { result } = renderHookWithClient(() => useSettingsSave())
    const failing = vi.fn().mockRejectedValue({ response: { status: 500 } })
    const other = vi.fn().mockResolvedValue(undefined)
    const later = vi.fn().mockResolvedValue(undefined)

    await act(() => result.current.save([[failing, other], [later]]))

    expect(other).toHaveBeenCalled()
    expect(later).toHaveBeenCalled()
    expect(showToast).toHaveBeenCalledTimes(1)
    expect(showToast).toHaveBeenCalledWith(expect.not.stringContaining('gesperrt'), 'error')
  })

  it('reports the ancestor lock instead of a generic error', async () => {
    const { result } = renderHookWithClient(() => useSettingsSave())
    const failing = vi.fn().mockRejectedValue({ response: { status: 409 } })

    await act(() => result.current.save([[failing]]))

    expect(showToast).toHaveBeenCalledWith(expect.stringContaining('gesperrt'), 'error')
  })

  it('starts a phase only after the one before has settled', async () => {
    const { result } = renderHookWithClient(() => useSettingsSave())
    const order: string[] = []
    let finishFirst: () => void = () => undefined
    const first = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          finishFirst = () => {
            order.push('first')
            resolve()
          }
        }),
    )
    const second = vi.fn(() => {
      order.push('second')
      return Promise.resolve()
    })

    let saving: Promise<void> = Promise.resolve()
    act(() => {
      saving = result.current.save([[first], [second]])
    })
    await waitFor(() => expect(first).toHaveBeenCalled())
    expect(second).not.toHaveBeenCalled()
    expect(result.current.pending).toBe(true)

    await act(async () => {
      finishFirst()
      await saving
    })
    expect(order).toEqual(['first', 'second'])
    expect(result.current.pending).toBe(false)
  })

  it('does nothing without a part to save', async () => {
    const { result } = renderHookWithClient(() => useSettingsSave())

    await act(() => result.current.save([[], []]))

    expect(showToast).not.toHaveBeenCalled()
  })
})
