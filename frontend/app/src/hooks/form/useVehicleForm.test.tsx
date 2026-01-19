import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useVehicleForm } from './useVehicleForm'
import { Toaster } from '@green-ecolution/ui'
import { VehicleType, DrivingLicense, VehicleStatus } from '@green-ecolution/backend-client'

vi.mock('@/api/backendApi', () => ({
  vehicleApi: {
    createVehicle: vi.fn(),
    updateVehicle: vi.fn(),
  },
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
}))

import { vehicleApi } from '@/api/backendApi'
import type { Vehicle } from '@green-ecolution/backend-client'

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
  type: VehicleType.VehicleTypeTransporter,
  drivingLicense: DrivingLicense.DrivingLicenseB,
  status: VehicleStatus.VehicleStatusAvailable,
  height: 2.5,
  width: 2.0,
  length: 6.0,
  weight: 3.5,
  waterCapacity: 1000,
  description: '',
}

describe('useVehicleForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('initializes form with provided default values', () => {
    const { result } = renderHook(() => useVehicleForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    expect(result.current.form.getValues('numberPlate')).toBe('HH-AB-1234')
    expect(result.current.form.getValues('model')).toBe('Mercedes Sprinter')
    expect(result.current.form.getValues('type')).toBe(VehicleType.VehicleTypeTransporter)
    expect(result.current.form.getValues('waterCapacity')).toBe(1000)
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
      id: 1,
      numberPlate: 'HH-AB-1234',
      model: 'Mercedes Sprinter',
    } as Vehicle
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
        type: VehicleType.VehicleTypeTransporter,
        drivingLicense: DrivingLicense.DrivingLicenseB,
        status: VehicleStatus.VehicleStatusAvailable,
        height: 2.5,
        width: 2.0,
        length: 6.0,
        weight: 3.5,
        waterCapacity: 1000,
        description: '',
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            numberPlate: 'HH-AB-1234',
            model: 'Mercedes Sprinter',
          }) as unknown,
        }),
      )
    })
  })

  it('calls updateVehicle API when mutationType is update', async () => {
    const mockResponse = {
      id: 5,
      numberPlate: 'HH-XY-5678',
      model: 'VW Crafter',
    } as Vehicle
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
        type: VehicleType.VehicleTypeTransporter,
        drivingLicense: DrivingLicense.DrivingLicenseB,
        status: VehicleStatus.VehicleStatusAvailable,
        height: 2.5,
        width: 2.0,
        length: 6.0,
        weight: 3.5,
        waterCapacity: 1000,
        description: '',
      })
    })

    await waitFor(() => {
      expect(updateMock).toHaveBeenCalledWith(
        expect.objectContaining({
          id: '5',
          body: expect.objectContaining({ numberPlate: 'HH-XY-5678' }) as unknown,
        }),
      )
    })
  })

  it('passes numeric fields correctly to API', async () => {
    const mockResponse = {
      id: 1,
      numberPlate: 'HH-AB-1234',
    } as Vehicle
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
        waterCapacity: 2000,
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            height: 3.0,
            width: 2.5,
            length: 7.0,
            weight: 4.0,
            waterCapacity: 2000,
          }) as unknown,
        }),
      )
    })
  })
})
