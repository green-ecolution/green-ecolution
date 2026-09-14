/**
 * Holds the plugin's api key in memory only. A restart forgets it, which is
 * intended: the key is issued once in the ui and pasted in again, so nothing
 * on disk can leak it.
 */
export const createSession = (client) => {
  let key = null
  let identity = null

  return {
    async connect(candidate) {
      const trimmed = typeof candidate === 'string' ? candidate.trim() : ''
      if (trimmed === '') {
        return { ok: false, status: 400, body: { error: 'api key is empty' } }
      }

      const { status, body } = await client.identity(trimmed)
      if (status !== 200) return { ok: false, status, body }

      key = trimmed
      identity = body
      return { ok: true, status, identity }
    },
    clear() {
      key = null
      identity = null
    },
    state() {
      return { connected: key !== null, identity }
    },
    key() {
      return key
    },
  }
}
