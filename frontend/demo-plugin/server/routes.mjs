import { DELETED_EXTERNAL_ID, importBatch, modifiedBatch } from './demoTrees.mjs'

const answer = (status, body) => ({ status, body })

export const handleApi = async (method, pathname, payload, session, client) => {
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
