import { describe, expect, it } from 'vitest'
import { pluginViewKind } from './pluginView'

describe('pluginViewKind', () => {
  it('renders the iframe for an external plugin with a target', () => {
    expect(
      pluginViewKind({ frontendMode: 'external', frontendTarget: 'https://plugin.example.com' }),
    ).toEqual({ kind: 'iframe', target: 'https://plugin.example.com' })
  })

  it('shows the proxied notice instead of an iframe, even with a target set', () => {
    expect(
      pluginViewKind({ frontendMode: 'proxied', frontendTarget: 'plugin-backend:8080' }),
    ).toEqual({ kind: 'proxied' })
  })

  it('shows the no-frontend notice for mode none', () => {
    expect(pluginViewKind({ frontendMode: 'none', frontendTarget: null })).toEqual({
      kind: 'unavailable',
    })
  })

  it('shows the no-frontend notice for external mode with no target', () => {
    expect(pluginViewKind({ frontendMode: 'external', frontendTarget: null })).toEqual({
      kind: 'unavailable',
    })
  })

  it('refuses a target on the apps own origin', () => {
    expect(
      pluginViewKind(
        { frontendMode: 'external', frontendTarget: 'https://app.example.com/plugin' },
        'https://app.example.com',
      ),
    ).toEqual({ kind: 'unsafeOrigin' })
  })

  it('allows the same host on a different port', () => {
    expect(
      pluginViewKind(
        { frontendMode: 'external', frontendTarget: 'https://app.example.com:8443' },
        'https://app.example.com',
      ),
    ).toEqual({ kind: 'iframe', target: 'https://app.example.com:8443' })
  })

  it('refuses rather than throwing on a malformed target', () => {
    expect(
      pluginViewKind(
        { frontendMode: 'external', frontendTarget: 'not a url' },
        'https://app.example.com',
      ),
    ).toEqual({ kind: 'unavailable' })
  })

  it('defaults to the browser origin when none is passed', () => {
    expect(
      pluginViewKind({ frontendMode: 'external', frontendTarget: window.location.origin }),
    ).toEqual({ kind: 'unsafeOrigin' })
  })
})
