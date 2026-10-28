import { describe, it, expect, beforeEach, vi } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'
import { moduleApi, type OrganizationModulesResponse } from '@/api/backendApi'
import { useModuleActivationDraft } from './useModuleActivationDraft'

const getOrganizationModules = vi.spyOn(moduleApi, 'getOrganizationModules')
const updateOrganizationModules = vi.spyOn(moduleApi, 'updateOrganizationModules')

const response = (overrides: Partial<OrganizationModulesResponse> = {}) =>
  ({
    enforcedBy: null,
    modules: [
      {
        key: 'irrigation',
        enabled: true,
        origin: 'inherited',
        source: { id: 'root' },
        fixedByAncestor: false,
      },
      { key: 'waste', enabled: false, origin: 'own', ownValue: false, fixedByAncestor: false },
    ],
    ...overrides,
  }) as unknown as OrganizationModulesResponse

const loaded = async (data: OrganizationModulesResponse = response()) => {
  getOrganizationModules.mockResolvedValue(data)
  updateOrganizationModules.mockResolvedValue(data)
  const hook = renderHookWithClient(() => useModuleActivationDraft('nord', { enabled: true }))
  await waitFor(() => expect(hook.result.current.loading).toBe(false))
  return hook
}

describe('useModuleActivationDraft', () => {
  beforeEach(() => vi.clearAllMocks())

  it('starts from the response and is clean', async () => {
    const { result } = await loaded()

    expect(getOrganizationModules).toHaveBeenCalledWith({ orgId: 'nord' })
    expect(result.current.dirty).toBe(false)
    expect(result.current.valid).toBe(true)
    expect(result.current.rows).toEqual([
      {
        key: 'irrigation',
        enabled: true,
        origin: 'inherited',
        sourceId: 'root',
        own: null,
        setBy: null,
        locked: false,
      },
      {
        key: 'waste',
        enabled: false,
        origin: 'own',
        sourceId: null,
        own: false,
        setBy: null,
        locked: false,
      },
    ])
  })

  it('becomes dirty when a switch is flipped and shows the new state', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn('irrigation', false))

    expect(result.current.dirty).toBe(true)
    expect(result.current.rows[0]).toMatchObject({ enabled: false, own: false })
    // The rows follow the draft, the activation in force follows the server.
    expect(result.current.isActive('irrigation')).toBe(true)
  })

  it('returns a module to inheritance with null', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn('waste', null))
    expect(result.current.dirty).toBe(true)
    expect(result.current.rows[1].own).toBeNull()

    await act(() => result.current.save())
    expect(updateOrganizationModules).toHaveBeenCalledWith({
      orgId: 'nord',
      organizationModulesUpdateRequest: { modules: { waste: null } },
    })
  })

  it('sends only the switches that changed', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn('waste', true))
    await act(() => result.current.save())

    expect(updateOrganizationModules).toHaveBeenCalledTimes(1)
    expect(updateOrganizationModules).toHaveBeenCalledWith({
      orgId: 'nord',
      organizationModulesUpdateRequest: { modules: { waste: true } },
    })
  })

  it('is clean again once a switch is back where the server has it', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn('waste', true))
    act(() => result.current.setOwn('waste', false))

    expect(result.current.dirty).toBe(false)
  })

  it('drops the edits on reset', async () => {
    const { result } = await loaded()

    act(() => result.current.setOwn('irrigation', false))
    act(() => result.current.reset())

    expect(result.current.dirty).toBe(false)
    expect(result.current.rows[0].own).toBeNull()
  })

  it('locks every row while an ancestor holds the lock', async () => {
    const { result } = await loaded(response({ enforcedBy: { id: 'root' } }))

    expect(result.current.enforcedBy).toEqual({ id: 'root' })
    expect(result.current.rows.every((row) => row.locked)).toBe(true)

    act(() => result.current.setOwn('irrigation', false))
    expect(result.current.dirty).toBe(false)
  })

  it('locks a switch a parent set for this organization and names the parent', async () => {
    const { result } = await loaded(
      response({
        modules: [
          {
            key: 'irrigation',
            enabled: true,
            origin: 'inherited',
            source: { id: 'root' },
            fixedByAncestor: false,
          },
          {
            key: 'waste',
            enabled: false,
            origin: 'own',
            ownValue: false,
            setBy: { id: 'root' },
            fixedByAncestor: true,
          },
        ],
      }),
    )

    expect(result.current.rows[1]).toMatchObject({ setBy: 'root', locked: true })
    expect(result.current.rows[0].locked).toBe(false)

    act(() => result.current.setOwn('waste', true))
    act(() => result.current.setOwn('waste', null))
    expect(result.current.dirty).toBe(false)
  })

  it('names the parent but keeps the switch open for whoever acts for it', async () => {
    const { result } = await loaded(
      response({
        modules: [
          {
            key: 'waste',
            enabled: false,
            origin: 'own',
            ownValue: false,
            setBy: { id: 'root' },
            fixedByAncestor: false,
          },
        ],
      }),
    )

    expect(result.current.rows[0]).toMatchObject({ setBy: 'root', locked: false })
  })

  it('does not name the organization itself as the one that set its switch', async () => {
    const { result } = await loaded(
      response({
        modules: [
          {
            key: 'waste',
            enabled: false,
            origin: 'own',
            ownValue: false,
            setBy: { id: 'nord' },
            fixedByAncestor: false,
          },
        ],
      }),
    )

    expect(result.current.rows[0].setBy).toBeNull()
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
