import { act, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { ChapterSection } from '@/lib/handbook/types'
import ChapterToc from './ChapterToc'

const SECTIONS: ChapterSection[] = [
  { anchor: 'board', title: 'Board', level: 2 },
  { anchor: 'status', title: 'Status', level: 3 },
  { anchor: 'route', title: 'Route', level: 2 },
]

// The hook resolves the headings out of the document by id, so the test has to
// put them there. jsdom never lays out, hence the stubbed rects.
function mountHeadings(tops: Record<string, number>) {
  for (const [id, top] of Object.entries(tops)) {
    const heading = document.createElement('h2')
    heading.id = id
    heading.dataset.testHeading = 'true'
    heading.getBoundingClientRect = () => ({ top }) as DOMRect
    document.body.append(heading)
  }
}

// The hook coalesces measurements into an animation frame, so every assertion
// about the active section has to wait one out.
const nextFrame = () =>
  act(async () => {
    await new Promise((resolve) => requestAnimationFrame(() => resolve(null)))
  })

const scrollAndSettle = async () => {
  window.dispatchEvent(new Event('scroll'))
  await nextFrame()
}

afterEach(() => {
  for (const heading of document.querySelectorAll('[data-test-heading]')) heading.remove()
  // Drops the router-style stub one test installs, so it cannot leak even when
  // an assertion in that test throws first.
  delete (window.history as { replaceState?: unknown }).replaceState
  window.history.replaceState(null, '', '/help/watering-plans')
})

describe('ChapterToc', () => {
  it('links every section by its anchor and indents subsections', () => {
    render(<ChapterToc sections={SECTIONS} />)

    expect(screen.getByRole('link', { name: 'Board' })).toHaveAttribute('href', '#board')
    expect(screen.getByRole('link', { name: 'Status' }).closest('li')).toHaveClass('pl-4')
  })

  it('renders nothing when the chapter has no sections', () => {
    const { container } = render(<ChapterToc sections={[]} />)

    expect(container).toBeEmptyDOMElement()
  })

  it('marks no section while the reader is still above the first heading', async () => {
    mountHeadings({ board: 300, status: 700, route: 1200 })
    render(<ChapterToc sections={SECTIONS} />)
    await nextFrame()

    for (const name of ['Board', 'Status', 'Route']) {
      expect(screen.getByRole('link', { name })).not.toHaveAttribute('aria-current')
    }
  })

  it('marks the last section whose heading passed the reading line', async () => {
    mountHeadings({ board: -400, status: 40, route: 900 })
    render(<ChapterToc sections={SECTIONS} />)
    await scrollAndSettle()

    expect(screen.getByRole('link', { name: 'Status' })).toHaveAttribute('aria-current', 'location')
    expect(screen.getByRole('link', { name: 'Board' })).not.toHaveAttribute('aria-current')
    expect(screen.getByRole('link', { name: 'Route' })).not.toHaveAttribute('aria-current')
  })

  it('puts the active section into the address bar without adding history entries', async () => {
    const before = window.history.length
    mountHeadings({ board: -400, status: 40, route: 900 })
    render(<ChapterToc sections={SECTIONS} />)
    await scrollAndSettle()

    expect(window.location.hash).toBe('#status')
    expect(window.history.length).toBe(before)
  })

  it('bypasses the history method the router overwrites, so scroll restoration stays out of it', async () => {
    const patched = vi.fn()
    // Mirrors what TanStack Router installs: an own property shadowing the
    // prototype, which notifies its subscribers on every call.
    Object.defineProperty(window.history, 'replaceState', {
      value: patched,
      configurable: true,
      writable: true,
    })
    mountHeadings({ board: -400, status: 40, route: 900 })
    render(<ChapterToc sections={SECTIONS} />)
    await scrollAndSettle()

    expect(patched).not.toHaveBeenCalled()
    expect(window.location.hash).toBe('#status')
  })

  it('keeps a deep link intact while the reader is above the first heading', async () => {
    window.history.replaceState(null, '', '/help/watering-plans#route')
    mountHeadings({ board: 300, status: 700, route: 1200 })
    render(<ChapterToc sections={SECTIONS} />)
    await scrollAndSettle()

    expect(window.location.hash).toBe('#route')
  })
})
