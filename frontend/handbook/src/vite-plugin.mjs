import { existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { generate } from './generate.mjs'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')

export function handbook({ language = 'de' } = {}) {
  return {
    name: 'gec:handbook',
    async buildStart() {
      await generate({ root, language })
    },
    configureServer(server) {
      // The PDF is rendered by `just handbook-pdf`, not by this plugin. Without
      // this guard a missing file falls through to the SPA fallback, and the
      // download button hands out index.html under a .pdf name.
      server.middlewares.use((req, res, next) => {
        const path = req.url?.split('?')[0] ?? ''
        if (!path.startsWith('/handbook/') || !path.endsWith('.pdf')) return next()
        if (existsSync(join(server.config.publicDir, path))) return next()

        res.statusCode = 404
        res.setHeader('Content-Type', 'text/plain; charset=utf-8')
        res.end('Handbook PDF not rendered yet — run `just handbook-pdf`.\n')
      })

      const watched = join(root, 'content')
      server.watcher.add(watched)

      // generate() removes the output tree before rewriting it, so two runs must
      // never overlap: an editor's "save all" fires one event per chapter and
      // would otherwise leave the dev server reading a half-written tree.
      let running = Promise.resolve()
      let debounce = null

      server.watcher.on('all', (_event, file) => {
        if (!file.startsWith(watched)) return
        clearTimeout(debounce)
        debounce = setTimeout(() => {
          running = running
            .then(() => generate({ root, language }))
            .then(() => server.ws.send({ type: 'full-reload' }))
            .catch((error) => server.config.logger.error(error.message))
        }, 50)
      })
    },
  }
}
