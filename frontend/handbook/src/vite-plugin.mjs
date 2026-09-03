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
      const watched = join(root, 'content')
      server.watcher.add(watched)
      server.watcher.on('all', (_event, file) => {
        if (!file.startsWith(watched)) return
        generate({ root, language })
          .then(() => server.ws.send({ type: 'full-reload' }))
          .catch((error) => server.config.logger.error(error.message))
      })
    },
  }
}
