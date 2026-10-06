import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { WateringStatus } from '@green-ecolution/backend-client'
import SensorPlacementCard from './SensorPlacementCard'
import type { Sensor } from '@/api/backendApi'

const { useQueryMock, requestActivateMock } = vi.hoisted(() => ({
  useQueryMock: vi.fn(),
  requestActivateMock: vi.fn(),
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

vi.mock('./SensorActionsContext', () => ({
  useSensorActions: () => ({ requestActivate: requestActivateMock }),
}))

const tree = {
  id: 'tree-1',
  number: '312',
  species: 'Acer platanoides',
  plantingYear: 2018,
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
    coordinate: { latitude: 54.78, longitude: 9.43 },
    linkedTreeId: 'tree-1',
    linkedClusterId: 'cluster-1',
    linkedClusterName: 'Friesischer Berg',
    ...overrides,
  }) as Sensor

const respondWith = ({ treeData = tree, clusterData = cluster } = {}) => {
  useQueryMock.mockImplementation(({ queryKey }: { queryKey: unknown[] }) => {
    if (queryKey[0] === 'treecluster') return { data: clusterData }
    return { data: treeData, isLoading: false, isError: false }
  })
}

describe('SensorPlacementCard', () => {
  beforeEach(() => {
    useQueryMock.mockReset()
    requestActivateMock.mockReset()
  })

  it('leads with the watering group and names the tree the sensor sits on', () => {
    respondWith()

    render(<SensorPlacementCard sensor={sensorWith({})} />)

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

    render(<SensorPlacementCard sensor={sensorWith({})} />)

    expect(screen.getAllByRole('link')[0]).toHaveTextContent('Friesischer Berg')
  })

  it('says so when the tree belongs to no watering group', () => {
    respondWith()

    render(
      <SensorPlacementCard
        sensor={sensorWith({ linkedClusterId: null, linkedClusterName: null })}
      />,
    )

    expect(screen.getByText('Der Baum gehört zu keiner Bewässerungsgruppe.')).toBeInTheDocument()
    expect(screen.getAllByRole('link')).toHaveLength(1)
  })

  it('offers to assign a tree when the sensor is not linked', () => {
    respondWith()

    render(
      <SensorPlacementCard
        sensor={sensorWith({ linkedTreeId: null, linkedClusterId: null, coordinate: null })}
      />,
    )

    expect(screen.getByText('Noch keine Verknüpfung')).toBeInTheDocument()
    expect(screen.queryByTestId('map-preview')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /Baum zuweisen/ }))
    expect(requestActivateMock).toHaveBeenCalledOnce()
  })
})
