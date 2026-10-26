import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import SensorPlacementCard from './SensorPlacementCard'
import type { Sensor } from '@/api/backendApi'
import type { ModuleManifest } from '@/modules/types'
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'

const { activeManifests, requestActivateMock } = vi.hoisted(() => ({
  activeManifests: { current: [] as ModuleManifest[] },
  requestActivateMock: vi.fn(),
}))

vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests.current, isActive: () => true }),
}))

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children, to }: { children: ReactNode; to: string }) => <a href={to}>{children}</a>,
}))

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children }: { children?: ReactNode }) => (
    <div data-testid="map-preview">{children}</div>
  ),
}))

vi.mock('@/components/map-gl/SensorMarker', () => ({
  default: () => <div data-testid="sensor-marker" />,
}))

vi.mock('./SensorActionsContext', () => ({
  useSensorActions: () => ({ requestActivate: requestActivateMock, canAssign: true }),
}))

const sensorWith = (overrides: Partial<Sensor>) =>
  ({
    id: 'eui-test',
    location: { latitude: 54.78, longitude: 9.43 },
    target: { module: 'probe', kind: 'probe', id: 'probe-1' },
    targetLabel: 'Messpunkt Nord',
    ...overrides,
  }) as Sensor

describe('SensorPlacementCard', () => {
  beforeEach(() => {
    requestActivateMock.mockReset()
    activeManifests.current = [probeTargetManifest('probe', [probeTarget('probe', 'probe')])]
  })

  it('renders the placement of the module that owns the target', async () => {
    render(<SensorPlacementCard sensor={sensorWith({})} />)

    expect(await screen.findByTestId('placement-probe')).toHaveTextContent('eui-test')
  })

  it('shows the core label without a link for a target of an inactive module', () => {
    activeManifests.current = []

    render(<SensorPlacementCard sensor={sensorWith({})} />)

    expect(screen.getByText('Messpunkt Nord')).toBeInTheDocument()
    expect(screen.queryByRole('link')).not.toBeInTheDocument()
    expect(screen.queryByTestId('placement-probe')).not.toBeInTheDocument()
  })

  it('shows the core label without a link for a target of an unknown module', () => {
    render(
      <SensorPlacementCard
        sensor={sensorWith({ target: { module: 'waste', kind: 'bin', id: 'b-1' } })}
      />,
    )

    expect(screen.getByText('Messpunkt Nord')).toBeInTheDocument()
    expect(screen.queryByRole('link')).not.toBeInTheDocument()
    expect(screen.queryByTestId('placement-probe')).not.toBeInTheDocument()
  })

  it('offers to assign a target when the sensor is not linked', () => {
    render(
      <SensorPlacementCard
        sensor={sensorWith({ target: null, targetLabel: null, location: null })}
      />,
    )

    expect(screen.getByText('Noch keine Verknüpfung')).toBeInTheDocument()
    expect(screen.queryByTestId('map-preview')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /Aktivieren/ }))
    expect(requestActivateMock).toHaveBeenCalledOnce()
  })
})
