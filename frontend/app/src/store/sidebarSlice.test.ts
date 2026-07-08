import { describe, it, expect, beforeEach } from 'vitest'
import useStore from './store'

describe('sidebarSlice', () => {
  beforeEach(() => {
    localStorage.clear()
    useStore.setState({ sidebarCollapsed: null })
  })

  it('defaults to null (no user choice)', () => {
    expect(useStore.getState().sidebarCollapsed).toBeNull()
  })

  it('stores the explicit user choice', () => {
    useStore.getState().setSidebarCollapsed(true)
    expect(useStore.getState().sidebarCollapsed).toBe(true)

    useStore.getState().setSidebarCollapsed(false)
    expect(useStore.getState().sidebarCollapsed).toBe(false)
  })

  it('persists only sidebarCollapsed to localStorage', () => {
    useStore.getState().setSidebarCollapsed(true)

    const raw = localStorage.getItem('green-ecolution-sidebar')
    expect(raw).not.toBeNull()

    const persisted = JSON.parse(raw!) as { state: Record<string, unknown> }
    expect(persisted.state).toEqual({ sidebarCollapsed: true })
  })
})
