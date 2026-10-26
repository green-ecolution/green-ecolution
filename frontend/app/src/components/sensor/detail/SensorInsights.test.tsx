import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { Sensor } from '@/api/backendApi'
import type { ModuleManifest } from '@/modules/types'
import { probeTargetManifest } from '@/test/targetManifest'
import SensorInsights from './SensorInsights'

const { activeManifests } = vi.hoisted(() => ({
  activeManifests: { current: [] as ModuleManifest[] },
}))

vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests.current, isActive: () => true }),
}))

const withInsights = (key: string): ModuleManifest => ({
  ...probeTargetManifest(key, []),
  sensorInsights: ({ sensor }) => <p data-testid={`insights-${key}`}>{sensor.id}</p>,
})

const sensor = (target: Sensor['target']) => ({ id: 'eui-1', target }) as Sensor

describe('SensorInsights', () => {
  beforeEach(() => {
    activeManifests.current = []
  })

  it('renders the insights of every active module for a sensor without target', () => {
    activeManifests.current = [withInsights('alpha'), withInsights('beta')]

    render(<SensorInsights sensor={sensor(null)} />)

    expect(screen.getByTestId('insights-alpha')).toHaveTextContent('eui-1')
    expect(screen.getByTestId('insights-beta')).toBeInTheDocument()
  })

  it('does not tie the insights to the module the target belongs to', () => {
    activeManifests.current = [withInsights('alpha')]

    render(<SensorInsights sensor={sensor({ module: 'beta', kind: 'bin', id: 'b-1' })} />)

    expect(screen.getByTestId('insights-alpha')).toBeInTheDocument()
  })

  it('renders nothing when no active module provides insights', () => {
    activeManifests.current = [probeTargetManifest('alpha', [])]

    const { container } = render(<SensorInsights sensor={sensor(null)} />)

    expect(container).toBeEmptyDOMElement()
  })
})
