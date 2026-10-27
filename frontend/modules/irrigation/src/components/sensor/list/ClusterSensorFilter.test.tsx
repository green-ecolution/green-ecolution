import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactElement } from 'react'

vi.mock('../../../api/queries', async () => {
  const actual =
    await vi.importActual<typeof import('../../../api/queries')>('../../../api/queries')
  return {
    ...actual,
    clusterQueries: {
      ...actual.clusterQueries,
      list: () => ({
        queryKey: ['clusters-filter-test'],
        queryFn: () =>
          Promise.resolve({
            data: [
              { id: 'cluster-1', name: 'Bahnhofstraße Nord' },
              { id: 'cluster-2', name: 'Hafenspitze' },
            ],
          }),
      }),
      detail: (id: string) => ({
        queryKey: ['cluster-filter-test', id],
        queryFn: () => Promise.resolve({ id, name: `Gruppe ${id}` }),
      }),
    },
  }
})

const { ClusterSensorFilterChips, ClusterSensorFilterControl } =
  await import('./ClusterSensorFilter')

const renderWithClient = (ui: ReactElement) =>
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      {ui}
    </QueryClientProvider>,
  )

describe('ClusterSensorFilter', () => {
  const onChange = vi.fn()
  beforeEach(() => {
    onChange.mockReset()
  })

  it('picking a group option reports the selected cluster', async () => {
    renderWithClient(<ClusterSensorFilterControl value={[]} onChange={onChange} />)

    fireEvent.click(screen.getByRole('button', { name: /^Gruppe/ }))
    fireEvent.click(await screen.findByRole('option', { name: 'Bahnhofstraße Nord' }))

    expect(onChange).toHaveBeenCalledExactlyOnceWith(['cluster-1'])
  })

  it('removing a group chip reports the selection without that cluster', async () => {
    renderWithClient(
      <ClusterSensorFilterChips value={['cluster-1', 'cluster-2']} onChange={onChange} />,
    )

    fireEvent.click(
      await screen.findByRole('button', { name: 'Filter Bahnhofstraße Nord entfernen' }),
    )

    expect(onChange).toHaveBeenCalledExactlyOnceWith(['cluster-2'])
  })

  it('names a selected cluster outside the first page by loading it', async () => {
    renderWithClient(<ClusterSensorFilterChips value={['cluster-99']} onChange={onChange} />)

    expect(
      await screen.findByRole('button', { name: 'Filter Gruppe cluster-99 entfernen' }),
    ).toBeInTheDocument()
  })
})
