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
import ChapterPager from './ChapterPager'

const visible = vi.hoisted(() => ({
  filter: (_slug: string): boolean => true,
}))

vi.mock('@/lib/handbook/useChapterVisibility', () => ({
  useChapterVisibility: () => visible.filter,
}))

function renderPager(slug: string) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const chapterRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/help/$slug',
    component: () => <ChapterPager slug={slug} />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([chapterRoute]),
    history: createMemoryHistory({ initialEntries: [`/help/${slug}`] }),
  })

  return render(
    <I18nextProvider i18n={getI18n()}>
      <RouterProvider router={router} />
    </I18nextProvider>,
  )
}

const pagerHrefs = () => screen.getAllByRole('link').map((link) => link.getAttribute('href'))

describe('ChapterPager', () => {
  beforeEach(() => {
    visible.filter = () => true
  })

  it('skips chapters hidden by an inactive module', async () => {
    visible.filter = (slug) => slug !== 'treecluster'
    renderPager('trees')

    expect(await screen.findByRole('navigation')).toBeInTheDocument()
    expect(pagerHrefs()).toEqual(['/help/map', '/help/watering-plans'])
  })

  it('still pages on from a hidden chapter reached by a shared link', async () => {
    visible.filter = (slug) => slug !== 'trees'
    renderPager('trees')

    expect(await screen.findByRole('navigation')).toBeInTheDocument()
    expect(pagerHrefs()).toEqual(['/help/map', '/help/treecluster'])
  })

  it('links to both neighbours of a chapter inside a part', async () => {
    renderPager('trees')

    expect(await screen.findByRole('navigation')).toBeInTheDocument()
    expect(pagerHrefs()).toEqual(['/help/map', '/help/treecluster'])
    expect(screen.getByRole('link', { name: /Die Karte/ })).toHaveAttribute('href', '/help/map')
    expect(screen.getByRole('link', { name: /Bewässerungsgruppen/ })).toHaveAttribute(
      'href',
      '/help/treecluster',
    )
  })

  it('names the part a step leads into', async () => {
    renderPager('sensor-installation')

    expect(await screen.findByText('Verwaltung')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Profil/ })).toHaveAttribute(
      'href',
      '/help/settings-profile',
    )
  })

  it('stays quiet about the part while the reader remains inside it', async () => {
    renderPager('trees')

    expect(await screen.findByRole('navigation')).toBeInTheDocument()
    expect(screen.queryByText('Grünflächen')).not.toBeInTheDocument()
  })

  it('offers only the successor on the first chapter', async () => {
    renderPager('introduction')

    expect(await screen.findByRole('navigation')).toBeInTheDocument()
    expect(pagerHrefs()).toEqual(['/help/getting-started'])
    expect(screen.getByRole('link', { name: /Erste Schritte/ })).toBeInTheDocument()
  })

  it('offers only the predecessor on the last chapter', async () => {
    renderPager('troubleshooting')

    expect(await screen.findByRole('navigation')).toBeInTheDocument()
    expect(pagerHrefs()).toEqual(['/help/glossary'])
    expect(screen.getByRole('link', { name: /Glossar/ })).toBeInTheDocument()
  })
})
