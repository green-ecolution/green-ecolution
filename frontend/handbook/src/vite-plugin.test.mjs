import { mkdtemp, mkdir, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { handbook } from './vite-plugin.mjs'

async function publicDirWithPdf(...names) {
  const dir = await mkdtemp(join(tmpdir(), 'handbook-public-'))
  await mkdir(join(dir, 'handbook'), { recursive: true })
  for (const name of names) await writeFile(join(dir, 'handbook', name), '%PDF-1.7\n')
  return dir
}

function pdfMiddleware(publicDir) {
  const registered = []
  handbook().configureServer({
    middlewares: { use: (fn) => registered.push(fn) },
    watcher: { add() {}, on() {} },
    ws: { send() {} },
    config: { publicDir, logger: { error() {} } },
  })

  return registered[0]
}

function call(middleware, url) {
  const res = {
    statusCode: 200,
    headers: {},
    body: undefined,
    setHeader(name, value) {
      this.headers[name.toLowerCase()] = value
    },
    end(body) {
      this.body = body
    },
  }
  let passedOn = false
  middleware({ url }, res, () => {
    passedOn = true
  })

  return { res, passedOn }
}

describe('handbook dev middleware', () => {
  it('answers a missing handbook pdf with 404 instead of the app shell', async () => {
    const middleware = pdfMiddleware(await publicDirWithPdf())

    const { res, passedOn } = call(middleware, '/handbook/green-ecolution-handbuch.pdf')

    expect(passedOn).toBe(false)
    expect(res.statusCode).toBe(404)
    expect(res.headers['content-type']).toMatch(/text\/plain/)
    expect(res.body).toContain('just handbook-pdf')
  })

  it('passes a rendered pdf on to the static handler', async () => {
    const middleware = pdfMiddleware(await publicDirWithPdf('green-ecolution-handbuch.pdf'))

    const { res, passedOn } = call(middleware, '/handbook/green-ecolution-handbuch.pdf')

    expect(passedOn).toBe(true)
    expect(res.statusCode).toBe(200)
  })

  it('ignores the query string when looking the pdf up', async () => {
    const middleware = pdfMiddleware(await publicDirWithPdf('green-ecolution-handbuch.pdf'))

    const { passedOn } = call(middleware, '/handbook/green-ecolution-handbuch.pdf?v=2')

    expect(passedOn).toBe(true)
  })

  it('leaves every other request alone', async () => {
    const middleware = pdfMiddleware(await publicDirWithPdf())

    expect(call(middleware, '/help/erste-schritte').passedOn).toBe(true)
    expect(call(middleware, '/handbook/').passedOn).toBe(true)
  })
})
