import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { Sensor } from '@/api/backendApi'
import TreePlacement from './TreePlacement'

const { useQueryMock } = vi.hoisted(() => ({
  useQueryMock: vi.fn(),
}))

vi.mock('@tanstack/react-query', () => ({
  useQuery: useQueryMock,
  queryOptions: (options: unknown) => options,
}))

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children, to }: { children: ReactNode; to: string }) => <a href={to}>{children}</a>,
}))

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: () => <div data-testid="map-preview" />,
}))

const tree = {
  id: 'tree-1',
  number: '312',
  species: 'Acer platanoides',
  plantingYear: 2018,
  treeClusterId: 'cluster-1' as string | null,
}

const cluster = {
  id: 'cluster-1',
  name: 'Friesischer Berg',
  wateringStatus: WateringStatus.Good,
  trees: [{}, {}, {}],
}

const sensorWith = (overrides: Partial<Sensor>) =>
  ({
    id: 'eui-test',
    location: { latitude: 54.78, longitude: 9.43 },
    target: { module: 'irrigation', kind: 'tree', id: 'tree-1' },
    targetLabel: 'Friesischer Berg',
    ...overrides,
  }) as Sensor

const respondWith = ({ treeData = tree, clusterData = cluster } = {}) => {
  useQueryMock.mockImplementation(({ queryKey }: { queryKey: unknown[] }) => {
    if (queryKey[0] === 'treecluster') return { data: clusterData }
    return { data: treeData, isLoading: false, isError: false }
  })
}

describe('TreePlacement', () => {
  beforeEach(() => {
    useQueryMock.mockReset()
  })

  it('leads with the watering group and names the tree the sensor sits on', () => {
    respondWith()

    render(<TreePlacement sensor={sensorWith({})} />)

    const links = screen.getAllByRole('link')
    expect(links[0]).toHaveTextContent('Friesischer Berg')
    expect(links[0]).toHaveTextContent('3 Bäume')
    expect(links[0]).toHaveAttribute('href', '/treecluster/$treeclusterId')
    expect(links[1]).toHaveTextContent('Baum-Nr. 312')
    expect(links[1]).toHaveTextContent('Acer platanoides')
    expect(screen.getByTestId('map-preview')).toBeInTheDocument()
  })

  it('shows the group name from the sensor before the group has loaded', () => {
    useQueryMock.mockImplementation(({ queryKey }: { queryKey: unknown[] }) =>
      queryKey[0] === 'treecluster'
        ? { data: undefined }
        : { data: tree, isLoading: false, isError: false },
    )

    render(<TreePlacement sensor={sensorWith({})} />)

    expect(screen.getAllByRole('link')[0]).toHaveTextContent('Friesischer Berg')
  })

  it('does not claim the tree has no group while the tree is still loading', () => {
    useQueryMock.mockImplementation(() => ({ data: undefined, isLoading: true, isError: false }))

    render(<TreePlacement sensor={sensorWith({ targetLabel: null })} />)

    expect(
      screen.queryByText('Der Baum gehört zu keiner Bewässerungsgruppe.'),
    ).not.toBeInTheDocument()
  })

  it('says so when the tree belongs to no watering group', () => {
    respondWith({ treeData: { ...tree, treeClusterId: null } })

    render(<TreePlacement sensor={sensorWith({ targetLabel: '312' })} />)

    expect(screen.getByText('Der Baum gehört zu keiner Bewässerungsgruppe.')).toBeInTheDocument()
    expect(screen.getAllByRole('link')).toHaveLength(1)
  })
})
