import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { queryRoots } from '@/api/queries'
import type { Sensor } from '@/api/backendApi'
import type { ModuleManifest } from '@/modules/types'
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'

const { activeManifests, activateSensor, setSensorTarget, invalidate } = vi.hoisted(() => ({
  activeManifests: { current: [] as ModuleManifest[] },
  activateSensor: vi.fn(),
  setSensorTarget: vi.fn(),
  invalidate: vi.fn(),
}))

vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests.current, isActive: () => true }),
}))

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  sensorApi: { activateSensor, setSensorTarget },
}))

vi.mock('@/lib/queryInvalidation', () => ({
  useInvalidateQueryRoots: () => invalidate,
}))

const { default: SensorActionsProvider, useSensorActions } = await import('./SensorActionsContext')

const Trigger = () => {
  const actions = useSensorActions()
  return (
    <>
      <button type="button" onClick={actions.requestActivate}>
        activate
      </button>
      <button type="button" onClick={actions.requestReassign}>
        reassign
      </button>
    </>
  )
}

const renderWith = (sensor: Partial<Sensor>) =>
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SensorActionsProvider sensor={{ id: 'eui-1', target: null, ...sensor } as Sensor}>
        <Trigger />
      </SensorActionsProvider>
    </QueryClientProvider>,
  )

const probe = probeTargetManifest('probe', [probeTarget('probe', 'probe')])
const waste = probeTargetManifest('waste', [
  probeTarget('waste', 'bin', { labelKey: 'sensor:entity.name' }),
])

describe('SensorActionsProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    activateSensor.mockResolvedValue({})
    setSensorTarget.mockResolvedValue({})
    activeManifests.current = [probe]
  })

  it('activates through the dialog of the only target kind and refreshes its module', async () => {
    renderWith({})

    fireEvent.click(screen.getByRole('button', { name: 'activate' }))
    fireEvent.click(await screen.findByRole('button', { name: 'activate-probe' }))

    await waitFor(() =>
      expect(activateSensor).toHaveBeenCalledWith({
        sensorId: 'eui-1',
        activateSensorRequest: { target: { module: 'probe', kind: 'probe', id: 'probe-1' } },
      }),
    )
    await waitFor(() =>
      expect(invalidate).toHaveBeenCalledWith([...queryRoots.sensor, ['probe-root']], {
        reloadRoutes: true,
      }),
    )
  })

  it('asks for the target kind first when several modules offer one', async () => {
    activeManifests.current = [probe, waste]
    renderWith({})

    fireEvent.click(screen.getByRole('button', { name: 'activate' }))
    expect(screen.queryByRole('button', { name: 'activate-probe' })).not.toBeInTheDocument()
    fireEvent.click(await screen.findByRole('button', { name: /^Sensor/ }))

    expect(await screen.findByRole('button', { name: 'activate-bin' })).toBeInTheDocument()
  })

  it('reassigns through the dialog of the current target kind', async () => {
    activeManifests.current = [probe, waste]
    renderWith({ target: { module: 'waste', kind: 'bin', id: 'b-0' } })

    fireEvent.click(screen.getByRole('button', { name: 'reassign' }))
    fireEvent.click(await screen.findByRole('button', { name: 'reassign-bin' }))

    await waitFor(() =>
      expect(setSensorTarget).toHaveBeenCalledWith({
        sensorId: 'eui-1',
        setSensorTargetRequest: { target: { module: 'waste', kind: 'bin', id: 'bin-1' } },
      }),
    )
  })
})
