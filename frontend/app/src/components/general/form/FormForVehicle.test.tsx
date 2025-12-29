import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForVehicle from './FormForVehicle'
import { vehicleSchema, VehicleForm } from '@/schema/vehicleSchema'
import { FormProvider, useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import ToastProvider from '@/context/ToastContext'
import { VehicleType, DrivingLicense, VehicleStatus } from '@green-ecolution/backend-client'

function TestWrapper({
  children,
  defaultValues,
}: {
  children: ReactNode
  defaultValues: VehicleForm
}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  const methods = useForm<VehicleForm>({
    defaultValues,
    resolver: zodResolver(vehicleSchema),
    mode: 'onChange',
  })

  return (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <FormProvider {...methods}>{children}</FormProvider>
      </ToastProvider>
    </QueryClientProvider>
  )
}

const defaultFormValues: VehicleForm = {
  numberPlate: '',
  model: '',
  type: VehicleType.VehicleTypeTransporter,
  drivingLicense: DrivingLicense.DrivingLicenseB,
  status: VehicleStatus.VehicleStatusAvailable,
  height: 0,
  width: 0,
  length: 0,
  weight: 0,
  waterCapacity: 0,
  description: '',
}

describe('FormForVehicle', () => {
  const mockOnSubmit = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders all form fields', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    expect(screen.getByLabelText(/kennzeichen/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/fahrzeugmodell/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/fahrzeugtyp/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/wasserkapazität/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/aktueller status/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/führerscheinklasse/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/höhe des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/breite des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/länge des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/gewicht des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('renders vehicle type select with options', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const typeSelect = screen.getByRole('combobox', { name: /fahrzeugtyp/i })
    const options = Array.from((typeSelect as HTMLSelectElement).options).map((opt) => opt.text)

    expect(options).toContain('Anhänger')
    expect(options).toContain('Transporter')
    expect(options).toContain('Unbekannt')
  })

  it('renders driving license select with options', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const licenseSelect = screen.getByRole('combobox', { name: /führerscheinklasse/i })
    const options = Array.from((licenseSelect as HTMLSelectElement).options).map((opt) => opt.text)

    expect(options).toContain('B')
    expect(options).toContain('BE')
    expect(options).toContain('C')
    expect(options).toContain('CE')
  })

  it('renders vehicle status select with options', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const statusSelect = screen.getByRole('combobox', { name: /aktueller status/i })
    const options = Array.from((statusSelect as HTMLSelectElement).options).map((opt) => opt.text)

    expect(options).toContain('Verfügbar')
    expect(options).toContain('Nicht Verfügbar')
    expect(options).toContain('Im Einsatz')
    expect(options).toContain('Unbekannt')
  })

  it('shows error message when displayError is true', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle
          displayError={true}
          errorMessage="Ein Fehler ist aufgetreten"
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/ein fehler ist aufgetreten/i)).toBeInTheDocument()
  })

  it('submit button is disabled when form is invalid', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const submitButton = screen.getByRole('button', { name: /speichern/i })
    expect(submitButton).toBeDisabled()
  })

  it('submit button is enabled when form is valid', async () => {
    const user = userEvent.setup()

    const validValues: VehicleForm = {
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

    render(
      <TestWrapper defaultValues={validValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const numberPlateInput = screen.getByLabelText(/kennzeichen/i)
    await user.click(numberPlateInput)
    await user.tab()

    await waitFor(() => {
      const submitButton = screen.getByRole('button', { name: /speichern/i })
      expect(submitButton).not.toBeDisabled()
    })
  })

  it('allows entering number plate and model', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const numberPlateInput = screen.getByLabelText(/kennzeichen/i)
    const modelInput = screen.getByLabelText(/fahrzeugmodell/i)

    await user.type(numberPlateInput, 'HH-XY-5678')
    await user.type(modelInput, 'VW Crafter')

    expect(numberPlateInput).toHaveValue('HH-XY-5678')
    expect(modelInput).toHaveValue('VW Crafter')
  })

  it('allows entering numeric values', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const waterCapacityInput = screen.getByLabelText(/wasserkapazität/i)
    await user.clear(waterCapacityInput)
    await user.type(waterCapacityInput, '1500')

    expect(waterCapacityInput).toHaveValue(1500)
  })

  it('allows selecting vehicle type', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} />
      </TestWrapper>,
    )

    const typeSelect = screen.getByLabelText(/fahrzeugtyp/i)
    await user.selectOptions(typeSelect, VehicleType.VehicleTypeTrailer)

    expect((typeSelect as HTMLSelectElement).value).toBe(VehicleType.VehicleTypeTrailer)
  })
})
