import { useSyncExternalStore } from 'react'

const keyFor = (userId: string) => `ge.organizationScope.${userId}`
const RELOAD_GUARD_KEY = 'ge.organizationScope.resetAt'
const RELOAD_GUARD_MS = 10_000

let scope: string | null = null
let activeUserId: string | null = null
const listeners = new Set<() => void>()

const emit = () => listeners.forEach((listener) => listener())

const setScope = (next: string | null) => {
  if (next === scope) return
  scope = next
  emit()
}

const readStored = (userId: string): string | null => {
  try {
    const stored = localStorage.getItem(keyFor(userId))
    return stored === '' ? null : stored
  } catch {
    return null
  }
}

export function getOrganizationScope(): string | null {
  return scope
}

export function useOrganizationScope(): string | null {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    getOrganizationScope,
    getOrganizationScope,
  )
}

export function loadOrganizationScope(userId: string): void {
  activeUserId = userId
  setScope(readStored(userId))
}

export function setOrganizationScope(userId: string, next: string | null): void {
  try {
    if (next) localStorage.setItem(keyFor(userId), next)
    else localStorage.removeItem(keyFor(userId))
  } catch {
    // the scope still applies for this page load
  }
  activeUserId = userId
  setScope(next)
}

// A stale scope would make every request fail, so drop it and reload once. The
// guard stops a loop if the backend keeps rejecting after the reset.
export function resetInvalidOrganizationScope(): void {
  if (activeUserId) setOrganizationScope(activeUserId, null)
  else setScope(null)

  try {
    const last = Number(sessionStorage.getItem(RELOAD_GUARD_KEY))
    if (last && Date.now() - last < RELOAD_GUARD_MS) return
    sessionStorage.setItem(RELOAD_GUARD_KEY, String(Date.now()))
  } catch {
    return
  }
  window.location.reload()
}
