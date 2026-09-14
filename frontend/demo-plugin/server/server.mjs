import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createGeClient } from './geClient.mjs'
import { createSession } from './session.mjs'
import { handleApi } from './routes.mjs'

const PORT = Number(process.env.PORT ?? 5175)
const API_BASE_URL = process.env.GE_API_BASE_URL ?? 'http://host.docker.internal:3030/api'
const DIST = fileURLToPath(new URL('../dist/', import.meta.url))
const MAX_BODY_BYTES = 8192

const MIME = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.ico': 'image/x-icon',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
}

const client = createGeClient({ baseUrl: API_BASE_URL })
const session = createSession(client)

const readJsonBody = (req) =>
  new Promise((resolve, reject) => {
    let raw = ''
    req.on('data', (chunk) => {
      raw += chunk
      if (raw.length > MAX_BODY_BYTES) reject(new Error('request body too large'))
    })
    req.on('end', () => {
      if (raw === '') return resolve(null)
      try {
        resolve(JSON.parse(raw))
      } catch {
        reject(new Error('request body is not json'))
      }
    })
    req.on('error', reject)
  })

const sendJson = (res, status, body) => {
  const payload = body === null ? '' : JSON.stringify(body)
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'content-length': Buffer.byteLength(payload),
  })
  res.end(payload)
}

const sendFile = async (res, pathname) => {
  const requested = normalize(join(DIST, pathname === '/' ? 'index.html' : pathname.slice(1)))
  const file = requested.startsWith(DIST) ? requested : join(DIST, 'index.html')

  try {
    const content = await readFile(file)
    res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' })
    res.end(content)
  } catch {
    const fallback = await readFile(join(DIST, 'index.html'))
    res.writeHead(200, { 'content-type': MIME['.html'] })
    res.end(fallback)
  }
}

// No X-Frame-Options and no frame-ancestors anywhere in here: the whole point
// of this service is to be embedded in Green Ecolution's iframe.
const server = createServer((req, res) => {
  const { pathname } = new URL(req.url, `http://localhost:${PORT}`)

  if (!pathname.startsWith('/api/')) {
    void sendFile(res, pathname)
    return
  }

  void (async () => {
    try {
      const payload = req.method === 'POST' ? await readJsonBody(req) : null
      const { status, body } = await handleApi(req.method, pathname, payload, session, client)
      sendJson(res, status, body)
    } catch (error) {
      sendJson(res, 400, { error: error.message })
    }
  })()
})

server.listen(PORT, () => {
  console.log(`demo-plugin listening on :${PORT}, talking to ${API_BASE_URL}`)
})
