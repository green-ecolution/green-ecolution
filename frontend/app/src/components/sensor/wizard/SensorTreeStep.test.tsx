import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import type React from 'react'
import type { GeolocationFix } from '@/hooks/useGeolocation'

vi.mock('@/components/geolocation/LocationMapPreview', () => ({
  default: () => <div data-testid="location-map-preview" />,
}))
vi.mock('@/components/geolocation/NearestTreeMapPreview', () => ({
  default: () => <div data-testid="nearest-tree-map-preview" />,
}))
vi.mock('@/components/sensor/SensorTreePickerSheet', () => ({
  default: () => null,
}))
vi.mock('@/api/queries', async () => {
  const actual = await vi.importActual<typeof import('@/api/queries')>('@/api/queries')
  return {
    ...actual,
    nearestTreeQuery: () => ({
      queryKey: ['nearest-test'],
      queryFn: () =>
        Promise.resolve({
          data: [
            {
              distanceMeters: 4,
              tree: {
                id: 'tree-1',
                number: '0815',
                species: 'Tilia cordata',
                latitude: 54.79,
                longitude: 9.43,
                sensor: null,
                treeClusterId: null,
              },
            },
          ],
        }),
    }),
    treeIdQuery: () => ({ queryKey: ['tree-test'], queryFn: () => Promise.resolve(null) }),
    treeClusterIdQuery: () => ({
      queryKey: ['cluster-test'],
      queryFn: () => Promise.resolve(null),
    }),
  }
})

import SensorTreeStep from './SensorTreeStep'

const fix: GeolocationFix = {
  latitude: 54.79,
  longitude: 9.43,
  accuracy: 7.5,
  altitude: null,
  altitudeAccuracy: null,
  heading: null,
  speed: null,
  timestamp: 1_710_000_000_000,
}

const renderStep = (props: Partial<React.ComponentProps<typeof SensorTreeStep>> = {}) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <SensorTreeStep
        position={fix}
        status="watching"
        errorMessage={null}
        selectedTreeId={null}
        onSelect={vi.fn()}
        onRelocate={vi.fn()}
        {...props}
      />
    </QueryClientProvider>,
  )
}

describe('SensorTreeStep GPS states', () => {
  it('shows a loading hint while GPS is still requesting (no fix yet)', () => {
    renderStep({ position: null, status: 'requesting' })
    expect(screen.getByText(/Standort wird ermittelt/i)).toBeInTheDocument()
    expect(screen.queryByTestId('nearest-tree-map-preview')).not.toBeInTheDocument()
  })

  it('shows the permission notice and keeps manual search reachable when GPS is denied', () => {
    renderStep({ position: null, status: 'denied' })
    expect(screen.getByText(/Standortzugriff verweigert/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /anderen baum auswählen/i })).toBeInTheDocument()
  })

  it('shows accuracy badge, relocate link and nearby trees once a fix exists', async () => {
    renderStep({ position: fix, status: 'watching' })
    expect(screen.getByRole('button', { name: /standort aktualisieren/i })).toBeInTheDocument()
    expect(await screen.findByText('Tilia cordata')).toBeInTheDocument()
  })
})
