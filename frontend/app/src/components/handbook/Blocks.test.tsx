/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - Tests use ad-hoc routes not in the generated route tree
import { render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import Blocks from './Blocks'
import type { Block } from '@/lib/handbook/types'

function renderBlocks(blocks: Block[]) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })

  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <Blocks blocks={blocks} />,
  })

  const helpRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/help/$slug',
    component: () => <div data-testid="help-page">Help</div>,
  })

  const mapRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/map',
    component: () => <div data-testid="map-page">Map</div>,
  })

  const routeTree = rootRoute.addChildren([indexRoute, helpRoute, mapRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  return render(<RouterProvider router={router} />)
}

describe('Blocks', () => {
  it('renders a heading with its anchor as id', async () => {
    renderBlocks([{ kind: 'heading', level: 2, text: 'Board', anchor: 'board' }])

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Board', level: 2 })).toHaveAttribute(
        'id',
        'board',
      )
    })
  })

  it('renders inline emphasis and technical values', async () => {
    renderBlocks([
      {
        kind: 'paragraph',
        children: [
          { kind: 'strong', children: [{ kind: 'text', value: 'Art' }] },
          { kind: 'text', value: ' ist ' },
          { kind: 'code', value: 'Tilia' },
        ],
      },
    ])

    await waitFor(() => {
      expect(screen.getByText('Art').tagName).toBe('STRONG')
    })
    expect(screen.getByText('Tilia').tagName).toBe('CODE')
  })

  it('renders steps as an ordered list and bullets as an unordered one', async () => {
    const items = [[{ kind: 'text', value: 'Eins' } as const]]

    const steps = renderBlocks([{ kind: 'steps', items }])
    await waitFor(() => {
      expect(steps.container.querySelector('ol')).not.toBeNull()
    })
    steps.unmount()

    const bullets = renderBlocks([{ kind: 'list', items }])
    await waitFor(() => {
      expect(bullets.container.querySelector('ul')).not.toBeNull()
    })
  })

  it('renders a callout as an alert', async () => {
    renderBlocks([
      {
        kind: 'callout',
        tone: 'warning',
        children: [{ kind: 'paragraph', children: [{ kind: 'text', value: 'Achtung.' }] }],
      },
    ])

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent('Achtung.')
    })
  })

  it('renders a figure with its caption', async () => {
    renderBlocks([{ kind: 'figure', image: 'map-overview.png', caption: 'Die Karte' }])

    await waitFor(() => {
      expect(screen.getByRole('img', { name: 'Die Karte' })).toBeInTheDocument()
    })
    expect(screen.getByText('Die Karte')).toBeInTheDocument()
  })

  it('renders a table with a header row', async () => {
    renderBlocks([
      {
        kind: 'table',
        head: [[{ kind: 'text', value: 'Status' }]],
        rows: [[[{ kind: 'text', value: 'Aktiv' }]]],
      },
    ])

    await waitFor(() => {
      expect(screen.getByRole('columnheader', { name: 'Status' })).toBeInTheDocument()
    })
    expect(screen.getByRole('cell', { name: 'Aktiv' })).toBeInTheDocument()
  })

  it('renders the three link kinds', async () => {
    renderBlocks([
      {
        kind: 'paragraph',
        children: [
          {
            kind: 'link',
            target: { kind: 'external', href: 'https://a.de' },
            children: [{ kind: 'text', value: 'extern' }],
          },
          {
            kind: 'link',
            target: { kind: 'chapter', slug: 'trees', anchor: 'liste' },
            children: [{ kind: 'text', value: 'kapitel' }],
          },
          {
            kind: 'link',
            target: { kind: 'app', to: '/map' },
            children: [{ kind: 'text', value: 'karte' }],
          },
        ],
      },
    ])

    await waitFor(() => {
      expect(screen.getByRole('link', { name: 'extern' })).toHaveAttribute('href', 'https://a.de')
    })
    expect(screen.getByRole('link', { name: 'kapitel' })).toHaveAttribute(
      'href',
      '/help/trees#liste',
    )
    expect(screen.getByRole('link', { name: 'karte' })).toHaveAttribute('href', '/map')
  })
})
