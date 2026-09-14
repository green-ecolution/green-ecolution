import { render, screen, waitFor } from '@testing-library/react'
import { PluginProvider } from '@green-ecolution/plugin-interface'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from './App'

const context = {
  ns: 'green-ecolution',
  v: 1,
  type: 'ge:init',
  payload: {
    locale: 'de' as const,
    theme: 'light' as const,
    user: { displayName: 'Test Nutzer' },
    plugin: { slug: 'demo-plugin' },
  },
}

const completeHandshake = () => {
  window.dispatchEvent(new MessageEvent('message', { data: context, source: window.parent }))
}

const renderApp = () =>
  render(
    <PluginProvider>
      <App />
    </PluginProvider>,
  )

// A fresh Response per call: a body can only be read once, and StrictMode runs
// the session effect twice.
const stubFetch = (body: unknown) =>
  vi.stubGlobal(
    'fetch',
    vi.fn(() =>
      Promise.resolve(
        new Response(JSON.stringify(body), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        }),
      ),
    ),
  )

beforeEach(() => {
  stubFetch({ connected: false, identity: null })
})

describe('App', () => {
  it('shows nothing until the host answers the handshake', () => {
    renderApp()

    expect(screen.queryByText(/Test Nutzer/)).not.toBeInTheDocument()
  })

  it('greets the operator with the context the host sent', async () => {
    renderApp()
    completeHandshake()

    expect(await screen.findByText(/Test Nutzer/)).toBeInTheDocument()
    expect(screen.getByText('demo-plugin')).toBeInTheDocument()
  })

  it('asks for the api key while no session exists', async () => {
    renderApp()
    completeHandshake()

    expect(await screen.findByLabelText(/API-Key/)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Import/ })).not.toBeInTheDocument()
  })

  it('offers the four actions once a session exists', async () => {
    stubFetch({ connected: true, identity: { slug: 'demo-plugin', enabled: true } })

    renderApp()
    completeHandshake()

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Import/ })).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: /ndern/ })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /schen/ })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Referenzen/ })).toBeInTheDocument()
  })
})
