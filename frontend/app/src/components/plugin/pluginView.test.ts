import { describe, expect, it } from 'vitest'
import { iframeAllow, pluginViewKind } from './pluginView'

describe('pluginViewKind', () => {
  it('renders the iframe for an external plugin with a target', () => {
    expect(
      pluginViewKind({ frontendMode: 'external', frontendTarget: 'https://plugin.example.com' }),
    ).toEqual({ kind: 'iframe', target: 'https://plugin.example.com' })
  })

  it('renders the iframe at the session url for a proxied plugin', () => {
    expect(
      pluginViewKind(
        {
          frontendMode: 'proxied',
          frontendTarget: 'kataster.plugins.svc:8080',
          frontendUrl: 'https://kataster.plugins.example.com/__ge/session?ticket=gev_x',
        },
        'https://app.example.com',
      ),
    ).toEqual({
      kind: 'iframe',
      target: 'https://kataster.plugins.example.com/__ge/session?ticket=gev_x',
    })
  })

  it('shows the proxied notice when the instance has no proxy', () => {
    expect(
      pluginViewKind({
        frontendMode: 'proxied',
        frontendTarget: 'kataster.plugins.svc:8080',
        frontendUrl: null,
      }),
    ).toEqual({ kind: 'proxied' })
  })

  it('refuses a proxied url on the apps own origin', () => {
    expect(
      pluginViewKind(
        {
          frontendMode: 'proxied',
          frontendTarget: 'x:80',
          frontendUrl: 'https://app.example.com/__ge/session',
        },
        'https://app.example.com',
      ),
    ).toEqual({ kind: 'unsafeOrigin' })
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

describe('iframeAllow', () => {
  it('lists granted capabilities as permission policy features', () => {
    expect(iframeAllow(['camera', 'bluetooth'])).toBe('camera; bluetooth')
  })

  it('is empty without capabilities', () => {
    expect(iframeAllow([])).toBe('')
  })

  it('drops values the app does not know', () => {
    expect(iframeAllow(['bluetooth', 'geolocation'])).toBe('bluetooth')
  })
})
