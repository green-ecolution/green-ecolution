import { describe, expect, it, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import type { TreeClusterInListResponse } from '@green-ecolution/backend-client'
import { renderHookWithClient } from '@/test/utils'
import { useShownStops } from './useShownStops'

const loadStopClusters = vi.fn<(stops: string[]) => Promise<TreeClusterInListResponse[]>>()
vi.mock('../api/plans', () => ({ loadStopClusters: (stops: string[]) => loadStopClusters(stops) }))

const cluster = (id: string) => ({ id }) as TreeClusterInListResponse

describe('useShownStops', () => {
  beforeEach(() => {
    loadStopClusters.mockImplementation((stops) =>
      Promise.resolve(stops.filter((id) => id !== 'hidden').map(cluster)),
    )
  })

  it('shows only the stops the reader can see', async () => {
    const { result } = renderHookWithClient(() => useShownStops(['c1', 'hidden', 'c2']))

    await waitFor(() => expect(result.current.shown).toEqual(['c1', 'c2']))
  })

  it('lists no stop before the first answer, but keeps them all on a change', () => {
    loadStopClusters.mockReturnValueOnce(new Promise(() => undefined))
    const { result } = renderHookWithClient(() => useShownStops(['c1', 'hidden']))

    expect(result.current.shown).toEqual([])
    expect(result.current.keepUnshown(['c3'])).toEqual(['c3', 'c1', 'hidden'])
  })

  it('shows a newly added stop while its cluster still loads', async () => {
    let stops = ['c1', 'hidden']
    const { result, rerender } = renderHookWithClient(() => useShownStops(stops))
    await waitFor(() => expect(result.current.shown).toEqual(['c1']))

    loadStopClusters.mockReturnValueOnce(new Promise(() => undefined))
    stops = ['c1', 'hidden', 'c3']
    rerender()

    expect(result.current.shown).toEqual(['c1', 'c3'])
  })

  it('keeps the unseen stops when the list reports a removal', async () => {
    const { result } = renderHookWithClient(() => useShownStops(['c1', 'hidden', 'c2']))
    await waitFor(() => expect(result.current.shown).toEqual(['c1', 'c2']))

    expect(result.current.keepUnshown(['c2'])).toEqual(['c2', 'hidden'])
  })
})
