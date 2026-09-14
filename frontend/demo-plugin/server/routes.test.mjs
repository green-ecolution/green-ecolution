import { describe, expect, it, vi } from 'vitest'
import { handleApi } from './routes.mjs'
import { createSession } from './session.mjs'
import { DELETED_EXTERNAL_ID, MODIFIED_EXTERNAL_ID, importBatch } from './demoTrees.mjs'

const clientStub = () => ({
  identity: vi.fn().mockResolvedValue({ status: 200, body: { slug: 'demo-plugin' } }),
  upsertTrees: vi.fn().mockResolvedValue({ status: 200, body: { summary: { created: 6 } } }),
  deleteTree: vi.fn().mockResolvedValue({ status: 204, body: null }),
  listRefs: vi.fn().mockResolvedValue({ status: 200, body: { items: [] } }),
})

const connected = async (client) => {
  const session = createSession(client)
  await session.connect('gep_abc.def')
  return session
}

describe('handleApi', () => {
  it('reports the session state', async () => {
    const client = clientStub()

    const result = await handleApi('GET', '/api/session', null, createSession(client), client)

    expect(result).toEqual({ status: 200, body: { connected: false, identity: null } })
  })

  it('accepts a key and answers with the new state', async () => {
    const client = clientStub()
    const session = createSession(client)

    const result = await handleApi('POST', '/api/session', { key: 'gep_abc.def' }, session, client)

    expect(result.status).toBe(200)
    expect(result.body).toEqual({ connected: true, identity: { slug: 'demo-plugin' } })
  })

  it('passes a rejected key through with the backend status', async () => {
    const client = clientStub()
    client.identity.mockResolvedValue({ status: 401, body: { error: 'invalid key' } })

    const result = await handleApi(
      'POST',
      '/api/session',
      { key: 'nope' },
      createSession(client),
      client,
    )

    expect(result).toEqual({ status: 401, body: { error: 'invalid key' } })
  })

  it('refuses an action while no key is configured', async () => {
    const client = clientStub()

    const result = await handleApi(
      'POST',
      '/api/actions/import',
      null,
      createSession(client),
      client,
    )

    expect(result.status).toBe(409)
    expect(client.upsertTrees).not.toHaveBeenCalled()
  })

  it('imports the full demo batch', async () => {
    const client = clientStub()
    const session = await connected(client)

    const result = await handleApi('POST', '/api/actions/import', null, session, client)

    expect(client.upsertTrees).toHaveBeenCalledWith('gep_abc.def', importBatch())
    expect(result).toEqual({ status: 200, body: { summary: { created: 6 } } })
  })

  it('sends only the modified tree for the modify action', async () => {
    const client = clientStub()
    const session = await connected(client)

    await handleApi('POST', '/api/actions/modify', null, session, client)

    const [, batch] = client.upsertTrees.mock.calls[0]
    expect(batch.items).toHaveLength(1)
    expect(batch.items[0].external_id).toBe(MODIFIED_EXTERNAL_ID)
  })

  it('deletes the designated tree', async () => {
    const client = clientStub()
    const session = await connected(client)

    const result = await handleApi('POST', '/api/actions/delete', null, session, client)

    expect(client.deleteTree).toHaveBeenCalledWith('gep_abc.def', DELETED_EXTERNAL_ID)
    expect(result).toEqual({ status: 204, body: null })
  })

  it('lists the plugin references', async () => {
    const client = clientStub()
    const session = await connected(client)

    const result = await handleApi('GET', '/api/actions/refs', null, session, client)

    expect(client.listRefs).toHaveBeenCalledWith('gep_abc.def')
    expect(result.body).toEqual({ items: [] })
  })

  it('answers an unknown route with 404', async () => {
    const client = clientStub()

    const result = await handleApi('GET', '/api/nope', null, createSession(client), client)

    expect(result.status).toBe(404)
  })
})
