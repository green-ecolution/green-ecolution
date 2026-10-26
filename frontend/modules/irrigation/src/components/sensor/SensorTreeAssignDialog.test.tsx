import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type { SensorResponse, TreeResponse } from '@green-ecolution/backend-client'

const tree = {
  id: 'tree-1',
  number: '0815',
  species: 'Tilia cordata',
  latitude: 54.79,
  longitude: 9.43,
  sensor: null,
} as unknown as TreeResponse

vi.mock('../../hooks/useTreeSearch', () => ({
  useTreeSearch: () => ({
    enabled: true,
    trimmed: '',
    items: [tree],
    total: 1,
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
    hasNextPage: false,
    fetchNextPage: vi.fn(),
    isFetchingNextPage: false,
  }),
}))

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children }: { children?: ReactNode }) => <div>{children}</div>,
}))
vi.mock('../map-gl/layers/useTreeMarkerLayer', () => ({ default: () => undefined }))
vi.mock('../map-gl/layers/useClusterBoundaryLayer', () => ({ default: () => undefined }))
vi.mock('@/components/map-gl/MapContext', () => ({ useMaplibreMap: () => ({ flyTo: vi.fn() }) }))
vi.mock('@/components/map-gl/useMapView', () => ({
  useMapCenter: () => [54.79, 9.43],
  useMapZoom: () => 13,
}))

const { default: SensorTreeAssignDialog } = await import('./SensorTreeAssignDialog')

const sensor = { id: 'eui-1', target: null, location: null } as unknown as SensorResponse

const renderDialog = (onConfirm = vi.fn(), error: string | null = null) =>
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SensorTreeAssignDialog
        open
        onOpenChange={vi.fn()}
        mode="activate"
        sensor={sensor}
        pending={false}
        error={error}
        onConfirm={onConfirm}
      />
    </QueryClientProvider>,
  )

describe('SensorTreeAssignDialog', () => {
  it('confirms the picked tree as a selected target', () => {
    const onConfirm = vi.fn()
    renderDialog(onConfirm)

    expect(screen.getByRole('button', { name: 'Aktivieren' })).toBeDisabled()
    fireEvent.click(screen.getByText('Tilia cordata'))
    fireEvent.click(screen.getByRole('button', { name: 'Aktivieren' }))

    expect(onConfirm).toHaveBeenCalledExactlyOnceWith({
      target: { module: 'irrigation', kind: 'tree', id: 'tree-1' },
      label: 'Baum 0815',
      detail: 'Tilia cordata',
    })
  })

  it('shows the error the core reports back', () => {
    renderDialog(vi.fn(), 'Sensor ist bereits einem Baum zugeordnet.')

    expect(screen.getByText('Sensor ist bereits einem Baum zugeordnet.')).toBeInTheDocument()
  })
})
