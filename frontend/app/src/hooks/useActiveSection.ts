import { useEffect, useState } from 'react'

// Mirrors the `scroll-mt-24` on rendered handbook headings (6rem): a section
// becomes the current one exactly when clicking its link would park its heading
// there, so scrolling and clicking never disagree about what is active.
const READING_LINE = 96

/**
 * The section the reader is currently in, or `null` while still above the first
 * heading. Anchors must be given in document order.
 */
export function useActiveSection(anchors: string[]): string | null {
  const [active, setActive] = useState<string | null>(null)

  // Keyed on the anchors themselves rather than the array: a caller that builds
  // the list inline would otherwise re-attach the listeners on every render.
  const key = anchors.join('\n')

  useEffect(() => {
    const ids = key === '' ? [] : key.split('\n')
    if (ids.length === 0) return

    let frame = 0

    const measure = () => {
      frame = 0
      const headings = ids
        .map((id) => document.getElementById(id))
        .filter((el): el is HTMLElement => el !== null)
      if (headings.length === 0) return

      // A short closing section never reaches the reading line, so at the end of
      // a scrollable page the last heading wins regardless of where it sits. The
      // scrollable check matters: without it a page shorter than the viewport
      // would report "at the bottom" from the start.
      const scrollable = document.documentElement.scrollHeight > window.innerHeight
      const atBottom =
        window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2
      if (scrollable && atBottom) {
        setActive(headings[headings.length - 1].id)
        return
      }

      const passed = headings.filter((el) => el.getBoundingClientRect().top <= READING_LINE + 1)
      setActive(passed.length === 0 ? null : passed[passed.length - 1].id)
    }

    const schedule = () => {
      if (frame === 0) frame = requestAnimationFrame(measure)
    }

    // Scheduled rather than measured on the spot: the first paint is also when
    // the browser jumps to a deep link's anchor, and a frame later the scroll
    // position is settled.
    schedule()
    window.addEventListener('scroll', schedule, { passive: true })
    window.addEventListener('resize', schedule)

    return () => {
      if (frame !== 0) cancelAnimationFrame(frame)
      window.removeEventListener('scroll', schedule)
      window.removeEventListener('resize', schedule)
    }
  }, [key])

  // Clamped on read instead of reset in the effect: moving to another chapter
  // swaps the anchor list, and the section held from the previous one must not
  // survive that.
  return active !== null && anchors.includes(active) ? active : null
}
