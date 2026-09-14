import { describe, expect, it, vi } from 'vitest'
import { createGeClient } from './geClient.mjs'

const jsonResponse = (status, body) =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } })

describe('createGeClient', () => {
  it('sends the key as a bearer token to the identity endpoint', async () => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse(200, { slug: 'demo-plugin' }))
    const client = createGeClient({ baseUrl: 'http://api.test/api', fetchImpl })

    const result = await client.identity('gep_abc.def')

    expect(fetchImpl).toHaveBeenCalledWith('http://api.test/api/v1/plugins/me', {
      method: 'GET',
      headers: { Authorization: 'Bearer gep_abc.def' },
    })
    expect(result).toEqual({ status: 200, body: { slug: 'demo-plugin' } })
  })

  it('posts an ingest batch as json', async () => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse(200, { summary: { created: 1 } }))
    const client = createGeClient({ baseUrl: 'http://api.test/api', fetchImpl })

    await client.upsertTrees('key', { items: [{ external_id: 'demo-001' }] })

    const [url, init] = fetchImpl.mock.calls[0]
    expect(url).toBe('http://api.test/api/v1/plugins/ingest/trees')
    expect(init.method).toBe('POST')
    expect(init.headers['Content-Type']).toBe('application/json')
    expect(JSON.parse(init.body)).toEqual({ items: [{ external_id: 'demo-001' }] })
  })

  it('escapes the external id when deleting and reports an empty body as null', async () => {
    const fetchImpl = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    const client = createGeClient({ baseUrl: 'http://api.test/api', fetchImpl })

    const result = await client.deleteTree('key', 'demo/006')

    expect(fetchImpl.mock.calls[0][0]).toBe(
      'http://api.test/api/v1/plugins/ingest/trees/demo%2F006',
    )
    expect(result).toEqual({ status: 204, body: null })
  })

  it('passes the limit through when listing references', async () => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse(200, { items: [] }))
    const client = createGeClient({ baseUrl: 'http://api.test/api', fetchImpl })

    await client.listRefs('key', 50)

    expect(fetchImpl.mock.calls[0][0]).toBe('http://api.test/api/v1/plugins/ingest/trees?limit=50')
  })

  it('reports a non-json answer as raw text instead of throwing', async () => {
    const fetchImpl = vi.fn().mockResolvedValue(new Response('<html>404</html>', { status: 404 }))
    const client = createGeClient({ baseUrl: 'http://api.test/api', fetchImpl })

    const result = await client.identity('key')

    expect(result).toEqual({ status: 404, body: { raw: '<html>404</html>' } })
  })

  it('strips a trailing slash from the base url', async () => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse(200, {}))
    const client = createGeClient({ baseUrl: 'http://api.test/api/', fetchImpl })

    await client.identity('key')

    expect(fetchImpl.mock.calls[0][0]).toBe('http://api.test/api/v1/plugins/me')
  })
})
