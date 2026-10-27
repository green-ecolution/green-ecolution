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

vi.mock('@/hooks/useGeolocation', () => ({
  default: () => ({
    status: 'watching' as const,
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
    sensorQueries: {
      ...actual.sensorQueries,
      detail: (id: string) => ({
        queryKey: ['sensor', id],
        queryFn: () =>
          Promise.resolve({
            id,
            status: 'prepared' as const,
            latestData: null,
            target: null,
          }),
      }),
    },
  }
})

const { activeManifests } = vi.hoisted(() => ({
  activeManifests: { current: [] as import('@/modules/types').ModuleManifest[] },
}))

vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests.current, isActive: () => true }),
}))

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
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'

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
  activeManifests.current = [probeTargetManifest('probe', [probeTarget('probe', 'probe')])]
})

const scanAndContinue = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.click(screen.getByText('__scan_now__'))
  await screen.findByText('Sensor erkannt')
  await user.click(screen.getByRole('button', { name: /^weiter/i }))
}

const pickProbeAndReview = async (user: ReturnType<typeof userEvent.setup>) => {
  await scanAndContinue(user)
  await user.click(await screen.findByRole('button', { name: 'pick-probe' }))
  await user.click(screen.getByRole('button', { name: /^weiter/i }))
  await screen.findByText('Zuordnung prüfen')
}

describe('Sensor wizard', () => {
  it('walks through scan → target → review → success with the module step', async () => {
    const user = userEvent.setup()
    renderRoute()

    expect(screen.getByText('Sensor-QR scannen')).toBeInTheDocument()
    await scanAndContinue(user)

    expect(await screen.findByTestId('wizard-step-probe')).toHaveTextContent('none')
    await user.click(screen.getByRole('button', { name: 'pick-probe' }))
    expect(screen.getByTestId('wizard-step-probe')).toHaveTextContent('probe-1')
    await user.click(screen.getByRole('button', { name: /^weiter/i }))

    await screen.findByText('Zuordnung prüfen')
    expect(screen.getByText('eui-a8404131af5e6451')).toBeInTheDocument()
    expect(screen.getByText('Probe 1')).toBeInTheDocument()
    expect(screen.getByText('probe detail')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /sensor aktivieren/i }))

    await waitFor(() => {
      expect(activateMock).toHaveBeenCalledWith({
        sensorId: 'eui-a8404131af5e6451',
        activateSensorRequest: { target: { module: 'probe', kind: 'probe', id: 'probe-1' } },
      })
    })

    await screen.findByText('Sensor aktiviert')
    expect(screen.getByText(/wurde probe 1 zugeordnet/)).toBeInTheDocument()
  })

  it('names the target step after the only target kind', () => {
    renderRoute()

    const stepper = screen.getByRole('navigation', { name: /fortschritt/i })
    expect(within(stepper).getByText('Bewässerung')).toBeInTheDocument()
  })

  it('asks for the target kind first when several modules offer one', async () => {
    activeManifests.current = [
      probeTargetManifest('probe', [probeTarget('probe', 'probe')]),
      probeTargetManifest('other', [
        probeTarget('other', 'bin', { labelKey: 'sensor:entity.name' }),
      ]),
    ]
    const user = userEvent.setup()
    renderRoute()

    await scanAndContinue(user)
    expect(screen.queryByTestId('wizard-step-probe')).not.toBeInTheDocument()
    await user.click(await screen.findByRole('button', { name: /^Sensor/ }))

    expect(await screen.findByTestId('wizard-step-bin')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'pick-bin' }))
    await user.click(screen.getByRole('button', { name: /^weiter/i }))
    await user.click(await screen.findByRole('button', { name: /sensor aktivieren/i }))

    await waitFor(() => {
      expect(activateMock).toHaveBeenCalledWith({
        sensorId: 'eui-a8404131af5e6451',
        activateSensorRequest: { target: { module: 'other', kind: 'bin', id: 'bin-1' } },
      })
    })
  })

  it('returns to step 1 with cleared state on "Nächsten Sensor scannen"', async () => {
    const user = userEvent.setup()
    renderRoute()

    await pickProbeAndReview(user)
    await user.click(screen.getByRole('button', { name: /sensor aktivieren/i }))
    await screen.findByText('Sensor aktiviert')

    await user.click(screen.getByRole('button', { name: /nächsten sensor/i }))
    expect(await screen.findByText('Sensor-QR scannen')).toBeInTheDocument()
  })

  it('blocks the stepper from skipping ahead after the sensor id is cleared', async () => {
    const user = userEvent.setup()
    renderRoute()

    await scanAndContinue(user)
    await user.click(await screen.findByRole('button', { name: 'pick-probe' }))

    const stepper = screen.getByRole('navigation', { name: /fortschritt/i })
    await user.click(within(stepper).getByRole('button', { name: /qr-scan/i }))
    await screen.findByText('Sensor erkannt')

    await user.click(screen.getByRole('button', { name: /anderen sensor scannen/i }))
    await screen.findByText('Sensor-QR scannen')

    expect(within(stepper).queryByRole('button', { name: /^bewässerung/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^weiter/i })).not.toBeInTheDocument()
  })

  it('keeps state on activation error and shows the error', async () => {
    activateMock.mockRejectedValueOnce(new Response('boom', { status: 409 }))
    const user = userEvent.setup()
    renderRoute()

    await pickProbeAndReview(user)
    await user.click(screen.getByRole('button', { name: /sensor aktivieren/i }))

    await screen.findByText('Sensor ist bereits verknüpft.')
    expect(screen.getByText('Zuordnung prüfen')).toBeInTheDocument()
  })
})
