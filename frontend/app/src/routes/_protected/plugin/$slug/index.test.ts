import { describe, it, expect, vi } from 'vitest'
import { isValidElement } from 'react'
import { QueryClient } from '@tanstack/react-query'
import { ResponseError } from '@green-ecolution/backend-client'
import { pluginApi, type PluginViewResponse } from '@/api/backendApi'

const { Route } = await import('./index')
const { default: Forbidden } = await import('@/components/layout/Forbidden')
const { default: EntityNotFound } = await import('@/components/layout/EntityNotFound')

interface LoaderOpts {
  context: { queryClient: QueryClient }
  params: { slug: string }
}
const loader = (opts: LoaderOpts) =>
  (Route.options.loader as (o: LoaderOpts) => Promise<{ crumb: { title: string } }>)(opts)

const errorComponent = (error: unknown) =>
  (Route.options.errorComponent as (props: { error: unknown; reset: () => void }) => unknown)({
    error,
    reset: () => undefined,
  })

describe('/plugin/$slug', () => {
  // The management endpoint needs plugin:read, which a user of the view
  // typically does not hold -- loading it here would deny everyone else.
  it('loads the view endpoint, not the managed plugin, and names the crumb', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const spy = vi.spyOn(queryClient, 'fetchQuery').mockResolvedValue({ name: 'Acme' })

    const data = await loader({ context: { queryClient }, params: { slug: 'acme' } })

    expect(spy).toHaveBeenCalledOnce()
    expect(spy.mock.calls[0][0]).toMatchObject({ queryKey: ['plugins', 'acme', 'view'] })
    expect(data).toEqual({ crumb: { title: 'Acme' } })
  })

  // A proxied view's frontend_url carries a one-time ticket; the cached
  // response of an earlier visit would replay a redeemed one.
  it('fetches a fresh view on every visit, even within the stale time', async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
    })
    const getPluginView = vi
      .spyOn(pluginApi, 'getPluginView')
      .mockResolvedValueOnce({ name: 'Acme', frontendUrl: 'ticket-1' } as PluginViewResponse)
      .mockResolvedValueOnce({ name: 'Acme', frontendUrl: 'ticket-2' } as PluginViewResponse)

    await loader({ context: { queryClient }, params: { slug: 'acme' } })
    await loader({ context: { queryClient }, params: { slug: 'acme' } })

    expect(getPluginView).toHaveBeenCalledTimes(2)
    expect(
      queryClient.getQueryData<PluginViewResponse>(['plugins', 'acme', 'view'])?.frontendUrl,
    ).toBe('ticket-2')
  })

  it('shows the forbidden page when the backend denies access', () => {
    const element = errorComponent(new ResponseError(new Response(null, { status: 403 })))
    expect(isValidElement(element) && element.type).toBe(Forbidden)
  })

  it('shows the not-found card for an unknown slug', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const element = errorComponent(new ResponseError(new Response(null, { status: 404 })))
    expect(isValidElement(element) && element.type).toBe(EntityNotFound)
  })
})
