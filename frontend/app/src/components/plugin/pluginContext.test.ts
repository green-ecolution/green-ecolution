import { describe, expect, it, vi } from 'vitest'

const getPluginView = vi.fn()
vi.mock('@/api/backendApi', () => ({ pluginApi: { getPluginView } }))

const { pluginContextFactory } = await import('./pluginContext')

const base = {
  locale: 'de' as const,
  theme: 'light' as const,
  user: { displayName: 'Jane Doe' },
  plugin: { slug: 'sensor-setup' },
}

describe('pluginContextFactory', () => {
  it('adds a freshly fetched view ticket to the context', async () => {
    getPluginView.mockResolvedValueOnce({ viewTicket: 'gev_one' })

    const context = await pluginContextFactory(base)()

    expect(getPluginView).toHaveBeenCalledWith({ pluginSlug: 'sensor-setup' })
    expect(context).toEqual({ ...base, viewTicket: 'gev_one' })
  })

  it('fetches a new ticket on every call', async () => {
    getPluginView.mockResolvedValueOnce({ viewTicket: 'gev_one' })
    getPluginView.mockResolvedValueOnce({ viewTicket: 'gev_two' })
    const factory = pluginContextFactory(base)

    await factory()
    const second = await factory()

    expect(second.viewTicket).toBe('gev_two')
  })
})
