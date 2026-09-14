const parse = (text) => {
  if (text === '') return null
  try {
    return JSON.parse(text)
  } catch {
    // A misconfigured GE_API_BASE_URL answers with html; reporting it raw keeps
    // the demo view useful instead of turning it into a 500.
    return { raw: text }
  }
}

export const createGeClient = ({ baseUrl, fetchImpl = fetch }) => {
  const root = baseUrl.replace(/\/$/, '')

  const call = async (method, path, key, body) => {
    const headers = { Authorization: `Bearer ${key}` }
    if (body !== undefined) headers['Content-Type'] = 'application/json'

    const init = { method, headers }
    if (body !== undefined) init.body = JSON.stringify(body)

    const response = await fetchImpl(`${root}${path}`, init)
    return { status: response.status, body: parse(await response.text()) }
  }

  return {
    identity: (key) => call('GET', '/v1/plugins/me', key),
    upsertTrees: (key, batch) => call('POST', '/v1/plugins/ingest/trees', key, batch),
    deleteTree: (key, externalId) =>
      call('DELETE', `/v1/plugins/ingest/trees/${encodeURIComponent(externalId)}`, key),
    listRefs: (key, limit = 100) => call('GET', `/v1/plugins/ingest/trees?limit=${limit}`, key),
  }
}
