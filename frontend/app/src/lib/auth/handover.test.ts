import { describe, it, expect, vi, afterEach } from 'vitest'
import { startSigninHandover, startSignoutHandover } from './handover'

// Mirrors a committed navigation: the document is gone, so nothing ever settles.
const never = () => new Promise<void>(() => undefined)

const visibility = (state: DocumentVisibilityState) => {
  Object.defineProperty(document, 'visibilityState', { value: state, configurable: true })
  document.dispatchEvent(new Event('visibilitychange'))
}

const settled = (promise: Promise<void>) =>
  Promise.race([promise.then(() => true), Promise.resolve().then(() => false)])

afterEach(() => {
  visibility('visible')
  vi.restoreAllMocks()
})

describe('startSigninHandover', () => {
  it('resolves when the redirect promise itself resolves without leaving the page', async () => {
    const session = { signinRedirect: vi.fn(() => Promise.resolve()) }

    await expect(startSigninHandover(session, '/map')).resolves.toBeUndefined()
    expect(session.signinRedirect).toHaveBeenCalledWith({ returnTo: '/map' })
  })

  it('resolves on a bfcache restore even though the redirect promise never settles', async () => {
    const session = { signinRedirect: vi.fn(never) }
    const handover = startSigninHandover(session, '/map')

    expect(await settled(handover)).toBe(false)

    window.dispatchEvent(new Event('pageshow'))

    await expect(handover).resolves.toBeUndefined()
  })

  it('resolves when the visitor returns from a closed custom tab', async () => {
    const session = { signinRedirect: vi.fn(never) }
    const handover = startSigninHandover(session, '/map')

    visibility('hidden')
    expect(await settled(handover)).toBe(false)

    visibility('visible')

    await expect(handover).resolves.toBeUndefined()
  })

  it('resolves instead of rejecting when the redirect fails', async () => {
    const session = { signinRedirect: vi.fn(() => Promise.reject(new Error('no authority'))) }

    await expect(startSigninHandover(session, '/map')).resolves.toBeUndefined()
  })

  it('stops listening once it has resolved', async () => {
    const remove = vi.spyOn(window, 'removeEventListener')
    const session = { signinRedirect: vi.fn(() => Promise.resolve()) }

    await startSigninHandover(session, '/map')

    expect(remove).toHaveBeenCalledWith('pageshow', expect.any(Function))
  })
})

describe('startSignoutHandover', () => {
  it('resolves on a bfcache restore even though the redirect promise never settles', async () => {
    const session = { signoutRedirect: vi.fn(never) }
    const handover = startSignoutHandover(session)

    expect(await settled(handover)).toBe(false)

    window.dispatchEvent(new Event('pageshow'))

    await expect(handover).resolves.toBeUndefined()
  })
})
