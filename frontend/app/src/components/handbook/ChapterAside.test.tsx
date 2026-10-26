import { afterEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from '@tanstack/react-router'
import type { ChapterSection } from '@/lib/handbook/types'
import ChapterAside from './ChapterAside'

vi.mock('@/lib/handbook/useChapterVisibility', () => ({
  useChapterVisibility: () => () => true,
}))

const SECTIONS: ChapterSection[] = [
  { anchor: 'board', title: 'Board', level: 2 },
  { anchor: 'route', title: 'Route', level: 2 },
]

const WIDE = '(min-width: 1024px)'

function matchOnly(matching: string | null) {
  vi.spyOn(window, 'matchMedia').mockImplementation((query: string) => ({
    matches: query === matching,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }))
}

function renderAside(sections: ChapterSection[] = SECTIONS) {
  const rootRoute = createRootRoute({ component: () => <ChapterAside sections={sections} /> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/help/watering-plans'] }),
  })
  return render(<RouterProvider router={router} />)
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('ChapterAside', () => {
  it('keeps the search in reach and folds the section list away on a narrow screen', async () => {
    matchOnly(null)

    const { container } = renderAside()

    expect(await screen.findByLabelText(/suchen/i)).toBeInTheDocument()
    const fold = container.querySelector('details')
    expect(fold).not.toBeNull()
    expect(fold).not.toHaveAttribute('open')
    expect(screen.getByRole('link', { name: 'Board' })).toHaveAttribute('href', '#board')
  })

  it('pins the search and the section list beside the chapter on a wide screen', async () => {
    matchOnly(WIDE)

    const { container } = renderAside()

    expect(await screen.findByLabelText(/suchen/i)).toBeInTheDocument()
    expect(container.querySelector('details')).toBeNull()
    expect(screen.getByRole('link', { name: 'Route' })).toHaveAttribute('href', '#route')
  })

  it('leaves out the fold for a chapter without sections', async () => {
    matchOnly(null)

    const { container } = renderAside([])

    expect(await screen.findByLabelText(/suchen/i)).toBeInTheDocument()
    expect(container.querySelector('details')).toBeNull()
  })
})
