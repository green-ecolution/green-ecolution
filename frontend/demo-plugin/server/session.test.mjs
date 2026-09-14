import { describe, expect, it, vi } from 'vitest'
import { createSession } from './session.mjs'

const clientWith = (response) => ({ identity: vi.fn().mockResolvedValue(response) })

describe('createSession', () => {
  it('starts without a key', () => {
    const session = createSession(clientWith({ status: 200, body: {} }))

    expect(session.state()).toEqual({ connected: false, identity: null })
    expect(session.key()).toBeNull()
  })

  it('keeps the key once the identity endpoint accepts it', async () => {
    const identity = { slug: 'demo-plugin', enabled: true }
    const session = createSession(clientWith({ status: 200, body: identity }))

    const result = await session.connect('  gep_abc.def  ')

    expect(result).toEqual({ ok: true, status: 200, identity })
    expect(session.key()).toBe('gep_abc.def')
    expect(session.state()).toEqual({ connected: true, identity })
  })

  it('rejects a key the backend refuses and stays unconnected', async () => {
    const session = createSession(clientWith({ status: 401, body: { error: 'invalid key' } }))

    const result = await session.connect('gep_wrong.key')

    expect(result).toEqual({ ok: false, status: 401, body: { error: 'invalid key' } })
    expect(session.state()).toEqual({ connected: false, identity: null })
  })

  it('refuses an empty key without calling the backend', async () => {
    const client = clientWith({ status: 200, body: {} })
    const session = createSession(client)

    const result = await session.connect('   ')

    expect(result.ok).toBe(false)
    expect(result.status).toBe(400)
    expect(client.identity).not.toHaveBeenCalled()
  })

  it('forgets the key on clear', async () => {
    const session = createSession(clientWith({ status: 200, body: { slug: 'demo-plugin' } }))
    await session.connect('gep_abc.def')

    session.clear()

    expect(session.state()).toEqual({ connected: false, identity: null })
    expect(session.key()).toBeNull()
  })
})
