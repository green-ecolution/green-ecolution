import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { I18nextProvider } from 'react-i18next'
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from '@tanstack/react-router'
import { getI18n } from '@/lib/i18n'
import { handbookIndex } from '@/lib/handbook'
import { Route as HelpIndexRoute } from './index'

const visible = vi.hoisted(() => ({ filter: (_slug: string): boolean => true }))

vi.mock('@/lib/handbook/useChapterVisibility', () => ({
  useChapterVisibility: () => visible.filter,
}))

function renderOverview() {
  const Overview = HelpIndexRoute.options.component!
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const overview = createRoute({
    getParentRoute: () => rootRoute,
    path: '/help',
    component: Overview,
  })
  const chapter = createRoute({
    getParentRoute: () => rootRoute,
    path: '/help/$slug',
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([overview, chapter]),
    history: createMemoryHistory({ initialEntries: ['/help'] }),
  })

  return render(
    <I18nextProvider i18n={getI18n()}>
      <RouterProvider router={router} />
    </I18nextProvider>,
  )
}

const total = Object.keys(handbookIndex.chapters).length
const hrefs = () => screen.getAllByRole('link').map((link) => link.getAttribute('href'))

describe('handbook overview', () => {
  beforeEach(() => {
    visible.filter = () => true
  })

  it('lists every chapter when none is hidden', async () => {
    renderOverview()

    expect(await screen.findByText(new RegExp(`^${total} Kapitel in`))).toBeInTheDocument()
    expect(hrefs()).toContain('/help/trees')
  })

  it('leaves out a hidden chapter and counts only the visible ones', async () => {
    visible.filter = (slug) => slug !== 'trees'
    renderOverview()

    expect(await screen.findByText(new RegExp(`^${total - 1} Kapitel in`))).toBeInTheDocument()
    expect(hrefs()).not.toContain('/help/trees')
  })

  it('starts reading at the first visible chapter', async () => {
    visible.filter = (slug) => slug !== 'introduction'
    renderOverview()

    const start = await screen.findByRole('link', { name: /Von vorne lesen/ })
    expect(start).toHaveAttribute('href', '/help/getting-started')
  })
})
