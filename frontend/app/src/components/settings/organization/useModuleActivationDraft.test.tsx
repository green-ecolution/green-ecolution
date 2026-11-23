import { describe, it, expect, beforeEach, vi } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import {
  moduleApi,
  type OrganizationModuleResponse,
  type OrganizationModulesResponse,
} from '@/api/backendApi'
import { useModuleActivationDraft } from './useModuleActivationDraft'

const getOrganizationModules = vi.spyOn(moduleApi, 'getOrganizationModules')
const updateOrganizationModules = vi.spyOn(moduleApi, 'updateOrganizationModules')

const modules = (rows: Partial<OrganizationModuleResponse>[]): OrganizationModulesResponse => ({
  modules: rows.map((row) => ({
    key: 'irrigation',
    enabled: true,
    switchable: true,
    ...row,
  })),
})

const serve = async (data: OrganizationModulesResponse) => {
  getOrganizationModules.mockResolvedValue(data)
  updateOrganizationModules.mockResolvedValue(data)
  const hook = renderHookWithClient(() => useModuleActivationDraft('nord', { enabled: true }))
  await waitFor(() => expect(hook.result.current.loading).toBe(false))
  return hook
}

const sent = (body: Record<string, boolean>) => ({
  orgId: 'nord',
  organizationModulesUpdateRequest: { modules: body },
})

describe('useModuleActivationDraft', () => {
  beforeEach(() => vi.clearAllMocks())

  it('lists every released module with its state and whether it is switchable', async () => {
    const { result } = await serve(
      modules([{ key: 'a' }, { key: 'b', enabled: false, switchable: false }]),
    )
    expect(result.current.rows).toEqual([
      { key: 'a', enabled: true, switchable: true },
      { key: 'b', enabled: false, switchable: false },
    ])
  })

  it('drafts a flip and reports the direction', async () => {
    const { result } = await serve(modules([{ key: 'a' }, { key: 'b', enabled: false }]))
    act(() => result.current.setEnabled('a', false))
    act(() => result.current.setEnabled('b', true))
    expect(result.current.isEnabled('a')).toBe(false)
    expect(result.current.isEnabled('b')).toBe(true)
    expect(result.current.enabling && result.current.disabling && result.current.dirty).toBe(true)
  })

  it('flipping back to the stored state is no change', async () => {
    const { result } = await serve(modules([{ key: 'a' }]))
    act(() => result.current.setEnabled('a', false))
    act(() => result.current.setEnabled('a', true))
    expect(result.current.dirty).toBe(false)
  })

  it('sends plain booleans, switching on and off in their own calls', async () => {
    const { result } = await serve(modules([{ key: 'a' }, { key: 'b', enabled: false }]))
    act(() => result.current.setEnabled('a', false))
    act(() => result.current.setEnabled('b', true))
    await act(() => result.current.saveEnabling())
    expect(updateOrganizationModules).toHaveBeenLastCalledWith(sent({ b: true }))
    await act(() => result.current.saveDisabling())
    expect(updateOrganizationModules).toHaveBeenLastCalledWith(sent({ a: false }))
    expect(updateOrganizationModules).toHaveBeenCalledTimes(2)
  })

  it('ignores a flip on a module the organization does not switch', async () => {
    const { result } = await serve(modules([{ key: 'a', switchable: false }]))
    act(() => result.current.setEnabled('a', false))
    expect(result.current.dirty).toBe(false)
  })

  it('drops the flips on reset and sends nothing when nothing changed', async () => {
    const { result } = await serve(modules([{ key: 'a' }]))
    act(() => result.current.setEnabled('a', false))
    act(() => result.current.reset())
    expect(result.current.isEnabled('a')).toBe(true)
    await act(() => result.current.saveDisabling())
    expect(updateOrganizationModules).not.toHaveBeenCalled()
  })

  it('asks nothing while disabled', () => {
    const { result } = renderHookWithClient(() =>
      useModuleActivationDraft('nord', { enabled: false }),
    )
    expect(getOrganizationModules).not.toHaveBeenCalled()
    expect(result.current.rows).toEqual([])
    expect(result.current.dirty).toBe(false)
  })
})
