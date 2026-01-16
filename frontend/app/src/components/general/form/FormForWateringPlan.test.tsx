import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForWateringPlan from './FormForWateringPlan'
import { wateringPlanSchema, WateringPlanForm } from '@/schema/wateringPlanSchema'
import { FormProvider, useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import ToastProvider from '@/context/ToastContext'
import {
  WateringPlanStatus,
  Vehicle,
  User,
  VehicleType,
  DrivingLicense,
  VehicleStatus,
} from '@green-ecolution/backend-client'

function TestWrapper({
  children,
  defaultValues,
}: {
  children: ReactNode
  defaultValues: WateringPlanForm
}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  const methods = useForm<WateringPlanForm>({
    defaultValues,
    resolver: zodResolver(wateringPlanSchema),
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

const futureDate = new Date()
futureDate.setDate(futureDate.getDate() + 7)

const defaultFormValues: WateringPlanForm = {
  date: futureDate,
  status: WateringPlanStatus.WateringPlanStatusPlanned,
  transporterId: -1,
  trailerId: -1,
  driverIds: [],
  clusterIds: [],
  description: '',
}

const mockTransporters = [
  {
    id: 1,
    numberPlate: 'HH-AB-1234',
    drivingLicense: DrivingLicense.DrivingLicenseB,
    type: VehicleType.VehicleTypeTransporter,
    status: VehicleStatus.VehicleStatusAvailable,
  },
  {
    id: 2,
    numberPlate: 'HH-XY-5678',
    drivingLicense: DrivingLicense.DrivingLicenseC,
    type: VehicleType.VehicleTypeTransporter,
    status: VehicleStatus.VehicleStatusAvailable,
  },
] as Vehicle[]

const mockTrailers = [
  {
    id: 10,
    numberPlate: 'HH-TR-0001',
    drivingLicense: DrivingLicense.DrivingLicenseBE,
    type: VehicleType.VehicleTypeTrailer,
    status: VehicleStatus.VehicleStatusAvailable,
  },
] as Vehicle[]

const mockUsers = [
  {
    id: '550e8400-e29b-41d4-a716-446655440000',
    firstName: 'Max',
    lastName: 'Mustermann',
    drivingLicenses: [DrivingLicense.DrivingLicenseB, DrivingLicense.DrivingLicenseC],
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    firstName: 'Anna',
    lastName: 'Schmidt',
    drivingLicenses: [DrivingLicense.DrivingLicenseB],
  },
] as User[]

describe('FormForWateringPlan', () => {
  const mockOnSubmit = vi.fn()
  const mockOnAddCluster = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders all form fields', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/datum des einsatzes/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/verknüpftes fahrzeug/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/verknüpfter anhänger/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/verknüpfte mitarbeitende/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('renders transporter select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const transporterSelect = screen.getByRole('combobox', { name: /verknüpftes fahrzeug/i })
    await user.click(transporterSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Kein Fahrzeug')).toBeInTheDocument()
    expect(within(listbox).getByText(/HH-AB-1234/)).toBeInTheDocument()
    expect(within(listbox).getByText(/HH-XY-5678/)).toBeInTheDocument()
  })

  it('renders trailer select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const trailerSelect = screen.getByRole('combobox', { name: /verknüpfter anhänger/i })
    await user.click(trailerSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Keinen Anhänger')).toBeInTheDocument()
    expect(within(listbox).getByText(/HH-TR-0001/)).toBeInTheDocument()
  })

  it('renders user select with options', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const userSelect = screen.getByRole('listbox', { name: /verknüpfte mitarbeitende/i })
    const options = Array.from((userSelect as HTMLSelectElement).options).map((opt) => opt.text)

    expect(options.some((opt) => opt.includes('Max Mustermann'))).toBe(true)
    expect(options.some((opt) => opt.includes('Anna Schmidt'))).toBe(true)
  })

  it('renders add cluster button', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByRole('button', { name: /bäume hinzufügen/i })).toBeInTheDocument()
  })

  it('calls onAddCluster when add cluster button is clicked', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const addButton = screen.getByRole('button', { name: /bäume hinzufügen/i })
    await user.click(addButton)

    expect(mockOnAddCluster).toHaveBeenCalled()
  })

  it('shows error message when displayError is true', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={true}
          errorMessage="Ein Fehler ist aufgetreten"
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/ein fehler ist aufgetreten/i)).toBeInTheDocument()
  })

  it('submit button is disabled when form is invalid', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const submitButton = screen.getByRole('button', { name: /speichern/i })
    expect(submitButton).toBeDisabled()
  })

  it('form validation is handled by schema', () => {
    // Form validity is controlled by react-hook-form with zodResolver
    // The schema tests (wateringPlanSchema.test.ts) verify all validation rules
    // This test confirms the form uses the correct validation configuration
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    // With invalid default values, submit should be disabled
    const submitButton = screen.getByRole('button', { name: /speichern/i })
    expect(submitButton).toBeDisabled()
  })

  it('allows selecting a transporter', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForWateringPlan
          displayError={false}
          transporters={mockTransporters}
          trailers={mockTrailers}
          users={mockUsers}
          onAddCluster={mockOnAddCluster}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const transporterSelect = screen.getByRole('combobox', { name: /verknüpftes fahrzeug/i })
    await user.click(transporterSelect)

    const listbox = await screen.findByRole('listbox')
    await user.click(within(listbox).getByText(/HH-AB-1234/))

    await waitFor(() => {
      expect(screen.getByRole('combobox', { name: /verknüpftes fahrzeug/i })).toHaveTextContent(
        /HH-AB-1234/i,
      )
    })
  })
})
