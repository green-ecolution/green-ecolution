import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { NEW_VEHICLE_DEFAULTS, useVehicleForm, vehicleCapacityDefaults } from './useVehicleForm'
import { manifestOf } from '@/test/moduleFixtures'
import { Toaster } from '@green-ecolution/ui'
import {
  VehicleType,
  DrivingLicense,
  VehicleAvailability,
  ResponseError,
} from '@green-ecolution/backend-client'
import useStore from '@/store/store'

vi.mock('@/api/backendApi', () => ({
  vehicleApi: {
    createVehicle: vi.fn(),
    updateVehicle: vi.fn(),
  },
}))

const mockUseBlocker = vi.fn().mockReturnValue({
  proceed: vi.fn(),
  reset: vi.fn(),
  status: 'idle',
})

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  useRouter: () => ({ invalidate: vi.fn().mockResolvedValue(undefined) }),
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  useBlocker: (...args: unknown[]) => mockUseBlocker(...args),
}))

import { vehicleApi } from '@/api/backendApi'
import type { Vehicle } from '@/api/backendApi'

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false, throwOnError: false },
    },
  })

  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster />
    </QueryClientProvider>
  )
}

const defaultInitForm = {
  numberPlate: 'HH-AB-1234',
  model: 'Mercedes Sprinter',
  type: VehicleType.Transporter,
  drivingLicense: DrivingLicense.B,
  availability: VehicleAvailability.Available,
  height: 2.5,
  width: 2.0,
  length: 6.0,
  weight: 3.5,
  capacities: { water: 1000 },
  description: '',
}

const offered = [
  ...(manifestOf('irrigation').capacities ?? []),
  ...(manifestOf('waste').capacities ?? []),
]

describe('useVehicleForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useStore.getState().clearAllFormDrafts()
  })

  afterEach(() => {
    useStore.getState().clearAllFormDrafts()
  })

  it('initializes form with provided default values', () => {
    const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    expect(result.current.form.getValues('numberPlate')).toBe('HH-AB-1234')
    expect(result.current.form.getValues('model')).toBe('Mercedes Sprinter')
    expect(result.current.form.getValues('type')).toBe(VehicleType.Transporter)
    expect(result.current.form.getValues('capacities.water')).toBe(1000)
  })

  it('returns form methods', () => {
    const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    expect(result.current.form).toBeDefined()
    expect(result.current.mutate).toBeDefined()
    expect(result.current.isError).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('calls createVehicle API when mutationType is create', async () => {
    const mockResponse = {
      id: 'vehicle-uuid-1',
      numberPlate: 'HH-AB-1234',
      model: 'Mercedes Sprinter',
    } as unknown as Vehicle
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(vehicleApi.createVehicle)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    act(() => {
      result.current.mutate({
        numberPlate: 'HH-AB-1234',
        model: 'Mercedes Sprinter',
        type: VehicleType.Transporter,
        drivingLicense: DrivingLicense.B,
        availability: VehicleAvailability.Available,
        height: 2.5,
        width: 2.0,
        length: 6.0,
        weight: 3.5,
        capacities: { water: 1000 },
        description: '',
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          vehicleCreateRequest: expect.objectContaining({
            numberPlate: 'HH-AB-1234',
            model: 'Mercedes Sprinter',
          }) as unknown,
        }),
      )
    })
  })

  it('calls updateVehicle API when mutationType is update', async () => {
    const mockResponse = {
      id: 'vehicle-uuid-5',
      numberPlate: 'HH-XY-5678',
      model: 'VW Crafter',
    } as unknown as Vehicle
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(vehicleApi.updateVehicle)
    updateMock.mockResolvedValueOnce(mockResponse)

    const updateInitForm = {
      ...defaultInitForm,
      numberPlate: 'HH-XY-5678',
      model: 'VW Crafter',
    }

    const { result } = renderHook(
      () => useVehicleForm('update', { vehicleId: '5', initForm: updateInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        numberPlate: 'HH-XY-5678',
        model: 'VW Crafter',
        type: VehicleType.Transporter,
        drivingLicense: DrivingLicense.B,
        availability: VehicleAvailability.Available,
        height: 2.5,
        width: 2.0,
        length: 6.0,
        weight: 3.5,
        capacities: { water: 1000 },
        description: '',
      })
    })

    await waitFor(() => {
      expect(updateMock).toHaveBeenCalledWith(
        expect.objectContaining({
          vehicleId: '5',
          vehicleUpdateRequest: expect.objectContaining({
            numberPlate: 'HH-XY-5678',
          }) as unknown,
        }),
      )
    })
  })

  it('passes numeric fields correctly to API', async () => {
    const mockResponse = {
      id: 'vehicle-uuid-1',
      numberPlate: 'HH-AB-1234',
    } as unknown as Vehicle
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(vehicleApi.createVehicle)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    act(() => {
      result.current.mutate({
        ...defaultInitForm,
        height: 3.0,
        width: 2.5,
        length: 7.0,
        weight: 4.0,
        capacities: { water: 2000 },
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          vehicleCreateRequest: expect.objectContaining({
            height: 3.0,
            width: 2.5,
            length: 7.0,
            weight: 4.0,
            capacities: { water: 2000 },
          }) as unknown,
        }),
      )
    })
  })

  it('sends a stored kind that is not offered back unchanged', async () => {
    const mockResponse = { id: 'vehicle-uuid-5', numberPlate: 'HH-AB-1234' } as unknown as Vehicle
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(vehicleApi.updateVehicle)
    updateMock.mockResolvedValueOnce(mockResponse)
    const initForm = {
      ...defaultInitForm,
      capacities: vehicleCapacityDefaults(offered, { water: 1000, sand: 12.5 }),
    }

    const { result } = renderHook(
      () => useVehicleForm('update', { vehicleId: '5', initForm, offered }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({ ...initForm, capacities: { ...initForm.capacities, water: 1500 } })
    })

    await waitFor(() => {
      expect(updateMock).toHaveBeenCalledWith(
        expect.objectContaining({
          vehicleUpdateRequest: expect.objectContaining({
            capacities: { water: 1500, sand: 12.5 },
          }) as unknown,
        }),
      )
    })
  })

  it('sends only the capacities that are set', async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(vehicleApi.createVehicle)
    createMock.mockResolvedValueOnce({ id: 'vehicle-uuid-6' } as unknown as Vehicle)

    const { result } = renderHook(
      () => useVehicleForm('create', { initForm: defaultInitForm, offered }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        ...defaultInitForm,
        capacities: { water: undefined, cargo_volume: 6000 },
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith({
        vehicleCreateRequest: expect.objectContaining({
          capacities: { cargo_volume: 6000 },
        }) as unknown,
      })
    })
  })

  it('creates a vehicle without capacities when no kind is offered', async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(vehicleApi.createVehicle)
    createMock.mockResolvedValueOnce({ id: 'vehicle-uuid-7' } as unknown as Vehicle)

    const { result } = renderHook(
      () => useVehicleForm('create', { initForm: defaultInitForm, offered: [] }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({ ...defaultInitForm, capacities: {} })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith({
        vehicleCreateRequest: expect.objectContaining({ capacities: {} }) as unknown,
      })
    })
  })

  it('starts with empty capacity fields', () => {
    expect(NEW_VEHICLE_DEFAULTS).not.toHaveProperty('capacities')
    expect(vehicleCapacityDefaults(offered)).toEqual({ water: undefined, cargo_volume: undefined })
  })

  describe('draft management', () => {
    it('clears draft on successful mutation', async () => {
      const mockResponse = {
        id: 'vehicle-uuid-1',
        numberPlate: 'HH-AB-1234',
      } as unknown as Vehicle
      // eslint-disable-next-line @typescript-eslint/unbound-method
      const createMock = vi.mocked(vehicleApi.createVehicle)
      createMock.mockResolvedValueOnce(mockResponse)

      useStore.getState().setFormDraft('vehicle-create', defaultInitForm)

      const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
        wrapper: createWrapper(),
      })

      expect(useStore.getState().formDrafts['vehicle-create']).toBeDefined()

      act(() => {
        result.current.mutate({ ...defaultInitForm })
      })

      await waitFor(() => {
        expect(useStore.getState().formDrafts['vehicle-create']).toBeUndefined()
      })
    })

    it('returns navigationBlocker with correct message', () => {
      const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
        wrapper: createWrapper(),
      })

      expect(result.current.navigationBlocker).toBeDefined()
      expect(result.current.navigationBlocker.message).toContain('Fahrzeug')
    })
  })

  it('surfaces a rejected save as a German message instead of crashing the page', async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(vehicleApi.updateVehicle)
    updateMock.mockRejectedValueOnce(
      new ResponseError(
        new Response(JSON.stringify({ error: 'height: invalid type: string' }), {
          status: 422,
          headers: { 'Content-Type': 'application/json' },
        }),
        'Response returned an error code',
      ),
    )

    const { result } = renderHook(
      () => useVehicleForm('update', { vehicleId: 'vehicle-uuid-1', initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate(defaultInitForm)
    })

    await waitFor(() => expect(result.current.isError).toBe(true))
    expect(result.current.error?.message).toBe(
      'Die eingegebenen Daten sind unvollständig oder ungültig.',
    )
  })
})
