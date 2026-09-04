import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { generate } from './generate.mjs'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')

try {
  const { index } = await generate({ root, language: 'de' })
  console.log(`handbook: generated ${Object.keys(index.chapters).length} chapters`)
} catch (error) {
  console.error(error.message)
  process.exit(1)
}
