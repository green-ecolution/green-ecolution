import { beforeAll, describe, expect, it } from 'vitest'
import type { TFunction } from 'i18next'
import { PluginFrontendDtoToJSON } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import { buildFrontendDto, frontendModeOptions, validateTarget } from './pluginFrontend'

let t: TFunction<'settings'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'settings')
})

describe('validateTarget', () => {
  describe('external', () => {
    it('accepts an absolute https url', () => {
      expect(validateTarget('external', 'https://plugin.example.com', t)).toBeNull()
    })

    it('rejects a non-https url', () => {
      expect(validateTarget('external', 'http://plugin.example.com', t)).not.toBeNull()
    })

    it('accepts http on localhost, the deliberate development exception', () => {
      expect(validateTarget('external', 'http://localhost:5173', t)).toBeNull()
    })

    it('accepts https on localhost with no port', () => {
      expect(validateTarget('external', 'https://localhost', t)).toBeNull()
    })

    it('rejects a relative path, since it is not an absolute url', () => {
      expect(validateTarget('external', '/plugin', t)).not.toBeNull()
    })

    it('rejects unparseable input without throwing', () => {
      expect(() => validateTarget('external', 'not a url at all', t)).not.toThrow()
      expect(validateTarget('external', 'not a url at all', t)).not.toBeNull()
    })

    it("rejects the app's own origin", () => {
      expect(validateTarget('external', window.location.origin, t)).not.toBeNull()
    })
  })

  describe('proxied', () => {
    it('accepts a bare host:port', () => {
      expect(validateTarget('proxied', 'svc.plugins.svc.cluster.local:8080', t)).toBeNull()
    })

    it('rejects a host with no port', () => {
      expect(validateTarget('proxied', 'svc.plugins.svc.cluster.local', t)).not.toBeNull()
    })

    it('rejects a non-numeric port', () => {
      expect(validateTarget('proxied', 'svc.plugins.svc.cluster.local:abc', t)).not.toBeNull()
    })

    it('rejects a bare IPv6 host with a port', () => {
      expect(validateTarget('proxied', '::1:8080', t)).not.toBeNull()
    })

    it('accepts an upper-case host, since DNS names ignore case', () => {
      expect(validateTarget('proxied', 'Demo-Plugin:80', t)).toBeNull()
    })

    it('accepts a label of exactly 63 characters', () => {
      expect(validateTarget('proxied', `${'a'.repeat(63)}.plugins.svc:8080`, t)).toBeNull()
    })

    // The backend's allowlist only compares plain DNS names: a URL parser
    // reads these characters as delimiters and would reach another host.
    it.each([
      ['a backslash', '169.254.169.254\\latest\\?.plugins.svc:8080'],
      ['a question mark', 'evil?x.plugins.svc:8080'],
      ['a hash', 'evil#x.plugins.svc:8080'],
      ['an at sign', 'user@evil.plugins.svc:8080'],
      ['an embedded colon', 'evil:1234.plugins.svc:8080'],
      ['a trailing dot', 'kataster.plugins.svc.:8080'],
      ['an empty label', 'kataster..plugins.svc:8080'],
      ['a label longer than 63 characters', `${'a'.repeat(64)}.plugins.svc:8080`],
      ['a label with a leading dash', '-kataster.plugins.svc:8080'],
      ['a label with a trailing dash', 'kataster-.plugins.svc:8080'],
      ['an underscore', 'kata_ster.plugins.svc:8080'],
      ['a host longer than 253 characters', `${`${'a'.repeat(60)}.`.repeat(5)}svc:8080`],
    ])('rejects a host with %s', (_, target) => {
      expect(validateTarget('proxied', target, t)).toBe(t('plugin.install.targetInvalidHostPort'))
    })
  })
})

/**
 * The generated client dispatches a `oneOf` by first matching guard, and those
 * guards only test which properties are present — so the variant order in the
 * OpenAPI document decides whether `target` survives serialization. It has been
 * dropped once already, silently. Regenerating the client cannot reintroduce
 * that without failing here.
 */
describe('buildFrontendDto through the generated serializer', () => {
  it('keeps the target of an external frontend', () => {
    expect(
      PluginFrontendDtoToJSON(buildFrontendDto('external', 'https://plugin.example.com')),
    ).toEqual({ mode: 'external', target: 'https://plugin.example.com' })
  })

  it('keeps the target of a proxied frontend', () => {
    expect(PluginFrontendDtoToJSON(buildFrontendDto('proxied', 'plugin-backend:8080'))).toEqual({
      mode: 'proxied',
      target: 'plugin-backend:8080',
    })
  })

  it('emits no target for a plugin without a frontend', () => {
    expect(PluginFrontendDtoToJSON(buildFrontendDto('none', ''))).toEqual({ mode: 'none' })
  })
})

describe('frontendModeOptions', () => {
  it('offers all three modes', () => {
    expect(frontendModeOptions(t).map((option) => option.value)).toEqual([
      'none',
      'external',
      'proxied',
    ])
  })
})
