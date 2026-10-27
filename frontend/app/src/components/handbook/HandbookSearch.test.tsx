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

const visible = vi.hoisted(() => ({
  filter: (_slug: string): boolean => true,
}))

vi.mock('@/lib/handbook/useChapterVisibility', () => ({
  useChapterVisibility: () => visible.filter,
}))

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

type Variant = 'page' | 'sidebar'

function buildHandbookSearchRouter(variant: Variant) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <HandbookSearch variant={variant} />,
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

function renderHandbookSearch(variant: Variant = 'page') {
  return render(<RouterProvider router={buildHandbookSearchRouter(variant)} />)
}

function renderHandbookSearchInStrictMode() {
  return render(
    <StrictMode>
      <RouterProvider router={buildHandbookSearchRouter('page')} />
    </StrictMode>,
  )
}

describe('HandbookSearch', () => {
  beforeEach(() => {
    mockedLoadSearchEntries.mockReset()
    visible.filter = () => true
  })

  const twoHits: SearchEntry[] = [
    {
      slug: 'watering-plans',
      anchor: 'route-festlegen',
      sectionTitle: 'Route festlegen',
      text: 'Erst die Gruppen wählen, dann die Route berechnen lassen.',
    },
    {
      slug: 'sensors',
      anchor: 'zustaende',
      sectionTitle: 'Zustände',
      text: 'Die Route eines Sensors spielt hier keine Rolle.',
    },
  ]

  it('leaves out hits from chapters hidden by an inactive module', async () => {
    mockedLoadSearchEntries.mockResolvedValue(twoHits)
    visible.filter = (slug) => slug !== 'watering-plans'
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'route')

    expect(await screen.findByRole('status')).toHaveTextContent('1 Treffer')
  })

  it('marks the searched term inside the excerpt', async () => {
    mockedLoadSearchEntries.mockResolvedValue(twoHits)
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'berechnen')

    const marks = await screen.findAllByText('berechnen')
    expect(marks.some((node) => node.tagName === 'MARK')).toBe(true)
  })

  it('announces how many results a query produced', async () => {
    mockedLoadSearchEntries.mockResolvedValue(twoHits)
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'route')

    expect(await screen.findByRole('status')).toHaveTextContent('2 Treffer')
  })

  it('clears the query and its results on escape', async () => {
    mockedLoadSearchEntries.mockResolvedValue(twoHits)
    const user = userEvent.setup()
    renderHandbookSearch()

    const field = await screen.findByLabelText(/suchen/i)
    await user.type(field, 'route')
    await screen.findByRole('link', { name: /Route festlegen/ })

    await user.type(field, '{Escape}')

    expect(screen.queryByRole('link', { name: /Route festlegen/ })).not.toBeInTheDocument()
    expect(field).toHaveValue('')
  })

  it('keeps the field labelled when it sits in the sidebar', async () => {
    mockedLoadSearchEntries.mockResolvedValue(twoHits)
    renderHandbookSearch('sidebar')

    expect(await screen.findByLabelText(/suchen/i)).toBeInTheDocument()
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

  it('links a chapter-level hit to the top of the chapter, without a dangling hash', async () => {
    const entries: SearchEntry[] = [
      { slug: 'glossary', anchor: '', sectionTitle: 'Glossar', text: 'Anwuchsfenster erklärt.' },
    ]
    mockedLoadSearchEntries.mockResolvedValue(entries)
    const user = userEvent.setup()
    renderHandbookSearch()

    await user.type(await screen.findByLabelText(/suchen/i), 'anwuchsfenster')

    await waitFor(() => {
      expect(screen.getByRole('link')).toBeInTheDocument()
    })
    expect(screen.getByRole('link')).toHaveAttribute('href', '/help/glossary')
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

    expect(await screen.findByRole('link', { name: /Route festlegen/ })).toBeInTheDocument()
  })
})
