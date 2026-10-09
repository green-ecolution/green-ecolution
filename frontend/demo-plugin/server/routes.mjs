import { DELETED_EXTERNAL_ID, importBatch, modifiedBatch } from './demoTrees.mjs'

const answer = (status, body) => ({ status, body })

// The proxy percent-encodes names (RFC 3986); a malformed escape must not turn the view into a 400.
const decodeName = (raw) => {
  try {
    return decodeURIComponent(raw)
  } catch {
    return raw
  }
}

export const handleApi = async (method, pathname, payload, session, client, headers = {}) => {
  if (pathname === '/api/session') {
    if (method === 'GET') return answer(200, session.state())
    if (method === 'POST') {
      const result = await session.connect(payload?.key)
      return result.ok ? answer(200, session.state()) : answer(result.status, result.body)
    }
    if (method === 'DELETE') {
      session.clear()
      return answer(200, session.state())
    }
  }

  if (pathname === '/api/proxy-identity' && method === 'GET') {
    const userId = headers['x-ge-user-id'] ?? null
    const rawName = headers['x-ge-user-name'] ?? null
    return answer(200, {
      proxied: userId !== null,
      userId,
      userName: rawName === null ? null : decodeName(rawName),
    })
  }

  if (pathname.startsWith('/api/actions/')) {
    const key = session.key()
    if (key === null) return answer(409, { error: 'no api key configured' })

    if (pathname === '/api/actions/import' && method === 'POST') {
      return await client.upsertTrees(key, importBatch())
    }
    if (pathname === '/api/actions/modify' && method === 'POST') {
      return await client.upsertTrees(key, modifiedBatch())
    }
    if (pathname === '/api/actions/delete' && method === 'POST') {
      return await client.deleteTree(key, DELETED_EXTERNAL_ID)
    }
    if (pathname === '/api/actions/refs' && method === 'GET') {
      return await client.listRefs(key)
    }
  }

  return answer(404, { error: 'unknown route' })
}
