import { describe, expect, it, vi, afterEach } from 'vitest'
import { cleanup, render, waitFor } from '@testing-library/react'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import Blocks from './Blocks'
import { handbookIndex, loadChapter } from '@/lib/handbook'
import type { Block } from '@/lib/handbook/types'

function renderBlocks(blocks: Block[]) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <Blocks blocks={blocks} />,
  })
  const chapterRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/help/$slug',
    component: () => <div data-testid="chapter-page" />,
  })
  const routeTree = rootRoute.addChildren([indexRoute, chapterRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
    // The router catches a render error in its own boundary instead of letting
    // it reach the test, so the boundary has to report it.
    defaultErrorComponent: ({ error }) => <p data-testid="render-error">{error.message}</p>,
  })

  return render(<RouterProvider router={router} />)
}

afterEach(cleanup)

describe('generated handbook chapters', () => {
  // types.ts is a hand-written mirror of the generator's output and the loader
  // casts rather than validates, so nothing else notices when a field is
  // renamed. Feeding the real chapters through the real renderer catches shape
  // drift, an unknown block kind and a missing image in one place.
  it('renders every generated chapter through the real block renderer', async () => {
    const slugs = Object.keys(handbookIndex.chapters)
    expect(slugs.length).toBeGreaterThan(0)

    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const failures: string[] = []

    for (const slug of slugs) {
      const content = await loadChapter(slug)
      try {
        expect(content.blocks.length).toBeGreaterThan(0)
        const view = renderBlocks(content.blocks)
        await waitFor(() => {
          expect(view.container.textContent?.trim()).not.toBe('')
        })
        expect(view.queryByTestId('render-error')?.textContent ?? null).toBeNull()
      } catch (error) {
        failures.push(`${slug}: ${error instanceof Error ? error.message : String(error)}`)
      } finally {
        cleanup()
      }
    }

    consoleError.mockRestore()
    expect(failures).toEqual([])
  })
})
