import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import type React from 'react'

vi.mock('@/components/scanner/QRScannerView', () => ({
  default: ({ onContinue }: { onContinue: (id: string) => void }) => (
    <button onClick={() => onContinue('A8404131AF5E6451')}>__scan_now__</button>
  ),
}))

vi.mock('@/components/geolocation/LocationMapPreview', () => ({
  default: () => <div data-testid="location-map-preview" />,
}))

vi.mock('@/components/geolocation/NearestTreeMapPreview', () => ({
  default: () => <div data-testid="nearest-tree-map-preview" />,
}))

vi.mock('@/components/geolocation/GPSStatusCard', () => ({
  default: () => <div data-testid="gps-status-card" />,
}))

vi.mock('@/components/geolocation/GeolocationPermissionNotice', () => ({
  default: () => <div data-testid="geolocation-permission-notice" />,
}))

vi.mock('@/components/sensor/SensorTreePickerSheet', () => ({
  default: () => null,
}))

vi.mock('@/hooks/useGeolocation', () => ({
  default: () => ({
    status: 'success' as const,
    position: {
      latitude: 54.79,
      longitude: 9.43,
      accuracy: 7.5,
      altitude: null,
      altitudeAccuracy: null,
      heading: null,
      speed: null,
      timestamp: 1_710_000_000_000,
    },
    history: [],
    errorMessage: null,
    start: vi.fn(),
    stop: vi.fn(),
    relocate: vi.fn().mockResolvedValue({
      latitude: 54.79,
      longitude: 9.43,
      accuracy: 7.5,
      altitude: null,
      altitudeAccuracy: null,
      heading: null,
      speed: null,
      timestamp: 1_710_000_000_000,
    }),
  }),
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
    treeIdQuery: () => ({
      queryKey: ['tree-test'],
      queryFn: () =>
        Promise.resolve({
          id: 'tree-1',
          number: '0815',
          species: 'Tilia cordata',
        }),
    }),
    treeClusterIdQuery: () => ({
      queryKey: ['cluster-test'],
      queryFn: () => Promise.resolve({ id: 'cluster-1', name: 'Test Cluster' }),
    }),
    sensorIdQuery: (id: string) => ({
      queryKey: ['sensor', id],
      queryFn: () =>
        Promise.resolve({
          id,
          status: 'prepared' as const,
          latestData: null,
          linkedTreeId: null,
        }),
    }),
  }
})

const activateMock = vi.fn().mockResolvedValue({ id: 'eui-a8404131af5e6451' })
vi.mock('@/api/backendApi', () => ({
  sensorApi: { activateSensor: (...args: unknown[]) => activateMock(...args) as unknown },
}))

const navigateMock = vi.fn()
vi.mock('@tanstack/react-router', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router')
  return {
    ...actual,
    useNavigate: () => navigateMock,
    createFileRoute: (_path: string) => (config: { component: () => React.JSX.Element }) => ({
      ...config,
      options: config,
    }),
    Link: ({ children, ...rest }: { children: React.ReactNode; [key: string]: unknown }) => (
      <a {...rest}>{children}</a>
    ),
  }
})

import { Route as NewSensorRoute } from '@/routes/_protected/sensors/new'

const renderRoute = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  const Component = (
    NewSensorRoute as unknown as { options: { component: () => React.JSX.Element } }
  ).options.component
  return render(
    <QueryClientProvider client={queryClient}>
      <Component />
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  activateMock.mockClear()
  navigateMock.mockClear()
})

describe('Sensor wizard', () => {
  it('walks through scan → gps → tree → review → success', async () => {
    const user = userEvent.setup()
    renderRoute()

    expect(screen.getByText('Sensor-QR scannen')).toBeInTheDocument()
    await user.click(screen.getByText('__scan_now__'))

    await screen.findByText(/bereit zur aktivierung/i)
    await user.click(screen.getByRole('button', { name: /^weiter/i }))

    await screen.findByText('Standort bestätigen')
    await user.click(screen.getByRole('button', { name: /^weiter/i }))

    await screen.findByText('Baum zuordnen')
    const treeItem = await screen.findByText('Tilia cordata')
    await user.click(treeItem)
    await user.click(screen.getByRole('button', { name: /^weiter/i }))

    await screen.findByText('Zuordnung prüfen')
    expect(screen.getByText('eui-a8404131af5e6451')).toBeInTheDocument()
    expect(screen.getByText('Tilia cordata')).toBeInTheDocument()
    expect(screen.getByText('0815')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /sensor aktivieren/i }))

    await waitFor(() => {
      expect(activateMock).toHaveBeenCalledWith({
        sensorId: 'eui-a8404131af5e6451',
        activateSensorRequest: { treeId: 'tree-1' },
      })
    })

    await screen.findByText('Sensor aktiviert')
  })

  it('returns to step 1 with cleared state on "Nächsten Sensor scannen"', async () => {
    const user = userEvent.setup()
    renderRoute()

    await user.click(screen.getByText('__scan_now__'))
    await screen.findByText(/bereit zur aktivierung/i)
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await screen.findByText('Standort bestätigen')
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await screen.findByText('Baum zuordnen')
    await user.click(await screen.findByText('Tilia cordata'))
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await user.click(await screen.findByRole('button', { name: /sensor aktivieren/i }))
    await screen.findByText('Sensor aktiviert')

    await user.click(screen.getByRole('button', { name: /nächsten sensor/i }))
    expect(await screen.findByText('Sensor-QR scannen')).toBeInTheDocument()
  })

  it('blocks the stepper from skipping ahead after the sensor id is cleared', async () => {
    const user = userEvent.setup()
    renderRoute()

    await user.click(screen.getByText('__scan_now__'))
    await screen.findByText(/bereit zur aktivierung/i)
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await screen.findByText('Standort bestätigen')
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await user.click(await screen.findByText('Tilia cordata'))

    const stepper = screen.getByRole('navigation', { name: /fortschritt/i })
    await user.click(within(stepper).getByRole('button', { name: /qr-scan/i }))
    await screen.findByText('Sensor erkannt')

    await user.click(screen.getByRole('button', { name: /anderen sensor scannen/i }))
    await screen.findByText('Sensor-QR scannen')

    expect(within(stepper).queryByRole('button', { name: /^gps/i })).not.toBeInTheDocument()
    expect(within(stepper).queryByRole('button', { name: /^baum/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^weiter/i })).not.toBeInTheDocument()
  })

  it('keeps state on activation error and shows the error', async () => {
    activateMock.mockRejectedValueOnce(new Response('boom', { status: 409 }))
    const user = userEvent.setup()
    renderRoute()

    await user.click(screen.getByText('__scan_now__'))
    await screen.findByText(/bereit zur aktivierung/i)
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await screen.findByText('Standort bestätigen')
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await user.click(await screen.findByText('Tilia cordata'))
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await user.click(await screen.findByRole('button', { name: /sensor aktivieren/i }))

    await screen.findByText(/bereits einem Baum/i)
    expect(screen.getByText('Zuordnung prüfen')).toBeInTheDocument()
  })
})
