import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import NearestTreeList from './NearestTreeList'
import type { TreeWithDistance } from '@/api/backendApi'

function makeEntry(
  overrides: {
    id?: string
    species?: string
    number?: string
    sensor?: unknown
    treeClusterId?: unknown
  } = {},
): TreeWithDistance {
  return {
    tree: {
      id: overrides.id ?? 'tree-1',
      species: overrides.species ?? 'Eiche',
      number: overrides.number ?? 'T-001',
      latitude: 0,
      longitude: 0,
      wateringStatus: 'unknown',
      treeClusterId: overrides.treeClusterId,
      sensor: overrides.sensor,
    } as TreeWithDistance['tree'],
    distanceMeters: 12,
  }
}

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('NearestTreeList', () => {
  it('disables items whose tree already has a sensor', () => {
    const entry = makeEntry({ id: 'tree-2', sensor: { id: 'eui-x' } })
    const onSelect = vi.fn()
    renderWithClient(<NearestTreeList trees={[entry]} selectedTreeId={null} onSelect={onSelect} />)

    const item = screen.getByRole('button', { name: /Eiche/ })
    expect(item).toHaveAttribute('aria-disabled', 'true')
    expect(item).toBeDisabled()
    expect(screen.getByText(/Sensor zugeordnet/i)).toBeInTheDocument()

    fireEvent.click(item)
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('auto-selects the first non-assigned tree on mount', () => {
    const onSelect = vi.fn()
    const entries = [
      makeEntry({ id: 'tree-a', sensor: { id: 'eui-x' } }),
      makeEntry({ id: 'tree-b' }),
    ]
    renderWithClient(<NearestTreeList trees={entries} selectedTreeId={null} onSelect={onSelect} />)

    expect(onSelect).toHaveBeenCalledWith('tree-b')
    expect(onSelect).not.toHaveBeenCalledWith('tree-a')
  })

  it('does not auto-select when every tree is assigned', () => {
    const onSelect = vi.fn()
    const entries = [
      makeEntry({ id: 'tree-a', sensor: { id: 'eui-x' } }),
      makeEntry({ id: 'tree-b', sensor: { id: 'eui-y' } }),
    ]
    renderWithClient(<NearestTreeList trees={entries} selectedTreeId={null} onSelect={onSelect} />)

    expect(onSelect).not.toHaveBeenCalled()
  })

  it('allows selecting an unassigned item', () => {
    const onSelect = vi.fn()
    const entries = [makeEntry({ id: 'tree-a' })]
    renderWithClient(<NearestTreeList trees={entries} selectedTreeId={null} onSelect={onSelect} />)

    const item = screen.getByRole('button', { name: /Eiche/ })
    expect(item).not.toBeDisabled()
    fireEvent.click(item)
    expect(onSelect).toHaveBeenCalledWith('tree-a')
  })
})
