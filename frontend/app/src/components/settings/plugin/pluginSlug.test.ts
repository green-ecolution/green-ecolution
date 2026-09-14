import { beforeAll, describe, expect, it } from 'vitest'
import type { TFunction } from 'i18next'
import { getI18n } from '@/lib/i18n'
import { RESERVED_SLUGS, validateSlug } from './pluginSlug'

let t: TFunction<'settings'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'settings')
})

describe('validateSlug', () => {
  it('accepts a well-formed slug', () => {
    expect(validateSlug('tbz-baumkataster', t)).toBeNull()
  })

  it('rejects an empty slug', () => {
    expect(validateSlug('', t)).not.toBeNull()
  })

  it('rejects uppercase and spaces', () => {
    expect(validateSlug('TBZ Kataster', t)).not.toBeNull()
  })

  it('rejects a leading hyphen', () => {
    expect(validateSlug('-kataster', t)).not.toBeNull()
  })

  it('rejects a slug longer than 64 characters', () => {
    expect(validateSlug('a'.repeat(65), t)).not.toBeNull()
  })

  it('rejects every slug the plugin routes reserve', () => {
    for (const reserved of RESERVED_SLUGS) {
      expect(validateSlug(reserved, t)).not.toBeNull()
    }
  })

  it('names the reserved slug in its message, so the admin knows which word to change', () => {
    expect(validateSlug('me', t)).toContain('me')
  })

  it('accepts a reserved segment used as a prefix', () => {
    expect(validateSlug('me-too', t)).toBeNull()
    expect(validateSlug('ingest', t)).toBeNull()
  })
})
