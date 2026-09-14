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

    /**
     * Documents current behaviour rather than prescribing it — see the report
     * for task 10's permission-matrix follow-up: the backend's
     * `PluginFrontendDto::into_domain` splits on the *last* ':' via
     * `str::rsplit_once`, so a bare (unbracketed) IPv6 host like `::1` followed
     * by `:8080` is accepted there (host `::1`, port `8080` — `ServiceEndpoint::new`
     * only rejects empty/too-long/slash-containing hosts, not colons). This
     * module's regex (`/^([^\s:]+):(\d+)$/`) instead forbids any colon in the
     * host segment, so the same input is rejected here. That is a genuine
     * client/backend disagreement, not something this test should paper over
     * by asserting a behaviour the code doesn't have.
     */
    it('rejects a bare IPv6 host with a port (diverges from the backend, see comment)', () => {
      expect(validateTarget('proxied', '::1:8080', t)).not.toBeNull()
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
  it('does not offer the proxied mode, since nothing serves such a view yet', () => {
    expect(frontendModeOptions(t).map((option) => option.value)).toEqual(['none', 'external'])
  })

  it('keeps the proxied mode for a plugin that already carries it', () => {
    // Otherwise renaming such a plugin would silently rewrite its frontend.
    expect(frontendModeOptions(t, 'proxied').map((option) => option.value)).toEqual([
      'none',
      'external',
      'proxied',
    ])
  })
})
