import { StrictMode } from 'react'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import HandbookSearch from './HandbookSearch'
import { loadSearchEntries } from '@/lib/handbook'
import type { SearchEntry } from '@/lib/handbook/types'

vi.mock('@/lib/handbook', () => ({
  handbookIndex: {
    parts: [],
    chapters: {
      glossary: {
        slug: 'glossary',
        title: 'Glossar',
        part: 'appendix',
        summary: '',
        routes: [],
        sections: [],
      },
    },
  },
  loadSearchEntries: vi.fn(),
}))

const mockedLoadSearchEntries = vi.mocked(loadSearchEntries)

function buildHandbookSearchRouter() {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <HandbookSearch />,
  })
  const chapterRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/help/$slug',
    component: () => <div data-testid="chapter-page" />,
  })
  const routeTree = rootRoute.addChildren([indexRoute, chapterRoute])
  return createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
}

function renderHandbookSearch() {
  return render(<RouterProvider router={buildHandbookSearchRouter()} />)
}

function renderHandbookSearchInStrictMode() {
  return render(
    <StrictMode>
      <RouterProvider router={buildHandbookSearchRouter()} />
    </StrictMode>,
  )
}

describe('HandbookSearch', () => {
  beforeEach(() => {
    mockedLoadSearchEntries.mockReset()
  })

  it('does not render an excerpt line for a title match on a section without paragraph text', async () => {
    const entries: SearchEntry[] = [
      { slug: 'glossary', anchor: 'begriffe', sectionTitle: 'Begriffe', text: '' },
    ]
    mockedLoadSearchEntries.mockResolvedValue(entries)
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'begriffe')

    await waitFor(() => {
      expect(screen.getByText('Begriffe')).toBeInTheDocument()
    })
    const item = screen.getByText('Begriffe').closest('a')
    expect(item?.querySelectorAll('span')).toHaveLength(2)
  })

  it('shows an error message instead of hanging silently when the search text fails to load', async () => {
    mockedLoadSearchEntries.mockRejectedValue(new Error('chunk load failed'))
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'begriffe')

    await waitFor(() => {
      expect(screen.getByText(/nicht verfügbar/i)).toBeInTheDocument()
    })
  })

  it('loads the search entries only once even while more searchable keystrokes arrive', async () => {
    let resolveLoad!: (entries: SearchEntry[]) => void
    mockedLoadSearchEntries.mockReturnValue(
      new Promise((resolve) => {
        resolveLoad = resolve
      }),
    )
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'begriffe')

    expect(mockedLoadSearchEntries).toHaveBeenCalledTimes(1)
    resolveLoad([])
  })

  it('still shows a result under StrictMode, whose phantom mount/cleanup must not leave the load results permanently discarded', async () => {
    const entries: SearchEntry[] = [
      {
        slug: 'watering-plans',
        anchor: 'route-festlegen',
        sectionTitle: 'Route festlegen',
        text: 'Erst die Gruppen wählen, dann die Route berechnen lassen.',
      },
    ]
    mockedLoadSearchEntries.mockResolvedValue(entries)
    const user = userEvent.setup()
    renderHandbookSearchInStrictMode()

    await user.type(await screen.findByLabelText(/suchen/i), 'route')

    await waitFor(() => {
      expect(screen.getByText('Route festlegen')).toBeInTheDocument()
    })
  })
})
