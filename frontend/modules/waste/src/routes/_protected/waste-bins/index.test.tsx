import type { ComponentType } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { z } from 'zod'
import { server } from '@/test/mocks/server'
import type { Permissions } from '@/lib/auth/permissions'
import { wasteHandlers } from '../../../test/handlers'
import { renderInRouter } from '../../../test/renderInRouter'

type Search = Record<string, unknown>
interface SearchUpdate {
  search: (prev: Search) => Search
  replace?: boolean
}

const { searchMock, navigate } = vi.hoisted(() => {
  const searchMock: { current: Search } = { current: { page: 1 } }
  return { searchMock, navigate: vi.fn((_: unknown) => Promise.resolve()) }
})

// The page reads search and navigation from its own route; the generated
// route tree is not loaded here, so the route object hands back test doubles.
vi.mock('@tanstack/react-router', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router')
  return {
    ...actual,
    createFileRoute: () => (options: Record<string, unknown>) => ({
      options,
      useSearch: () => searchMock.current,
      useNavigate: () => navigate,
    }),
  }
})

// The real session needs OIDC settings the test environment does not have.
vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

const permissions = vi.fn((): Permissions => new Set<string>())
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const { Route } = await import('./index')
const WasteBins = Route.options.component as ComponentType

const parseSearch = (input: Search) =>
  (Route.options.validateSearch as unknown as z.ZodType<Search>).parse(input)

const lastSearchUpdate = (prev: Search) => {
  const call = navigate.mock.lastCall?.[0] as SearchUpdate
  return call.search(prev)
}

describe('/waste-bins list', { timeout: 20_000 }, () => {
  beforeEach(() => {
    navigate.mockClear()
    searchMock.current = { page: 1 }
    permissions.mockReturnValue(new Set(['waste_bin:read']))
    server.use(...wasteHandlers)
  })

  it('renders a card per bin', async () => {
    renderInRouter(<WasteBins />)
    expect(await screen.findByText('B-0042')).toBeInTheDocument()
    expect(screen.getByText('B-0043')).toBeInTheDocument()
    expect(screen.getByText('Voll · 85 %')).toBeInTheDocument()
    expect(screen.getByText('Normal · 40 %')).toBeInTheDocument()
  })

  it('narrows the list by the fill state in the URL', async () => {
    searchMock.current = { page: 1, fillState: 'full' }
    renderInRouter(<WasteBins />)
    expect(await screen.findByText('B-0042')).toBeInTheDocument()
    expect(screen.queryByText('B-0043')).not.toBeInTheDocument()
  })

  it('writes the chosen fill state to the URL', async () => {
    const user = userEvent.setup()
    renderInRouter(<WasteBins />)
    await screen.findByText('B-0042')

    await user.click(screen.getByRole('button', { name: /Füllstufe/ }))
    await user.click(await screen.findByRole('option', { name: /Voll/ }))

    await waitFor(() => expect(navigate).toHaveBeenCalled())
    expect(lastSearchUpdate({ page: 3 })).toEqual({ page: 1, fillState: 'full' })
  })

  it('writes the search term to the URL', async () => {
    const user = userEvent.setup()
    renderInRouter(<WasteBins />)
    await screen.findByText('B-0042')

    await user.type(screen.getByRole('searchbox', { name: /Behälter suchen/ }), '42')

    await waitFor(() => expect(lastSearchUpdate({ page: 2 })).toEqual({ page: 1, q: '42' }))
  })

  it('offers creating a bin only with waste_bin:create', async () => {
    const { unmount } = renderInRouter(<WasteBins />)
    await screen.findByText('B-0042')
    expect(screen.queryByRole('link', { name: /Neuen Behälter erfassen/ })).not.toBeInTheDocument()
    unmount()

    permissions.mockReturnValue(new Set(['waste_bin:read', 'waste_bin:create']))
    renderInRouter(<WasteBins />)
    expect(await screen.findByRole('link', { name: /Neuen Behälter erfassen/ })).toHaveAttribute(
      'href',
      '/waste-bins/new',
    )
  })

  it('shows the filtered empty state with a reset', async () => {
    searchMock.current = { page: 1, q: 'nichts' }
    const user = userEvent.setup()
    renderInRouter(<WasteBins />)

    expect(await screen.findByText('Kein Behälter passt zu diesen Filtern.')).toBeInTheDocument()
    await user.click(screen.getAllByRole('button', { name: /Filter zurücksetzen/ })[0])
    expect(lastSearchUpdate({ page: 1, q: 'nichts' })).toEqual({
      page: 1,
      q: undefined,
      fillState: undefined,
    })
  })

  it('drops an unknown fill state from the URL instead of failing', () => {
    expect(parseSearch({ fillState: 'overflowing' }).fillState).toBeUndefined()
    expect(parseSearch({ fillState: 'full' }).fillState).toBe('full')
    expect(parseSearch({}).page).toBe(1)
  })
})
