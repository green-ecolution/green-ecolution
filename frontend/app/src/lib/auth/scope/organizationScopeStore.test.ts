import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getOrganizationScope,
  loadOrganizationScope,
  resetInvalidOrganizationScope,
  setOrganizationScope,
} from './organizationScopeStore'

const ORG = '0190a1b2-0000-7000-8000-000000000001'

describe('organizationScopeStore', () => {
  beforeEach(() => {
    localStorage.clear()
    sessionStorage.clear()
    loadOrganizationScope('nobody')
  })
  afterEach(() => vi.restoreAllMocks())

  it('writes and reads the scope under a per-user key', () => {
    setOrganizationScope('u1', ORG)
    expect(localStorage.getItem('ge.organizationScope.u1')).toBe(ORG)
    expect(getOrganizationScope()).toBe(ORG)

    loadOrganizationScope('nobody')
    expect(getOrganizationScope()).toBeNull()
    loadOrganizationScope('u1')
    expect(getOrganizationScope()).toBe(ORG)
  })

  it('loads null for another user', () => {
    setOrganizationScope('u1', ORG)
    loadOrganizationScope('u2')
    expect(getOrganizationScope()).toBeNull()
  })

  it('removes the stored value when set to null', () => {
    setOrganizationScope('u1', ORG)
    setOrganizationScope('u1', null)
    expect(localStorage.getItem('ge.organizationScope.u1')).toBeNull()
    expect(getOrganizationScope()).toBeNull()
  })

  it('stays null without error when localStorage throws', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('blocked')
    })
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('blocked')
    })
    expect(() => loadOrganizationScope('u1')).not.toThrow()
    expect(getOrganizationScope()).toBeNull()
    expect(() => setOrganizationScope('u1', ORG)).not.toThrow()
  })

  it('clears the scope and reloads once on an invalid scope', () => {
    const reload = vi.fn()
    vi.stubGlobal('location', { ...window.location, reload })
    setOrganizationScope('u1', ORG)

    resetInvalidOrganizationScope()
    expect(getOrganizationScope()).toBeNull()
    expect(localStorage.getItem('ge.organizationScope.u1')).toBeNull()
    expect(reload).toHaveBeenCalledTimes(1)

    setOrganizationScope('u1', ORG)
    resetInvalidOrganizationScope()
    expect(getOrganizationScope()).toBeNull()
    expect(reload).toHaveBeenCalledTimes(1)
    vi.unstubAllGlobals()
  })
})
