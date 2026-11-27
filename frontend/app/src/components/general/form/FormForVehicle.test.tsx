import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForVehicle from './FormForVehicle'
import { VehicleForm } from '@/schema/vehicleSchema'
import { FormProvider, useForm, useFormState } from 'react-hook-form'
import { vehicleDraftResolver } from '@/schema/vehicleSchema'
import { ReactNode } from 'react'
import { FORM_VALIDATION_MODE } from '@/hooks/form/useEntityForm'
import { NEW_VEHICLE_DEFAULTS, vehicleCapacityDefaults } from '@/hooks/form/useVehicleForm'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Toaster } from '@green-ecolution/ui'
import { VehicleType, DrivingLicense, VehicleAvailability } from '@green-ecolution/backend-client'
import type { ModuleCapacityKind } from '@/modules/types'
import { manifestOf } from '@/test/moduleFixtures'

const water = manifestOf('irrigation').capacities![0]
const cargo = manifestOf('waste').capacities![0]
let offered: ModuleCapacityKind[] = [water, cargo]

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
    resolver: vehicleDraftResolver<VehicleForm>((key) => key, offered),
    mode: FORM_VALIDATION_MODE,
  })

  return (
    <QueryClientProvider client={queryClient}>
      <FormProvider {...methods}>{children}</FormProvider>
      <Toaster />
    </QueryClientProvider>
  )
}

const defaultFormValues: VehicleForm = {
  numberPlate: '',
  model: '',
  type: VehicleType.Transporter,
  drivingLicense: DrivingLicense.B,
  availability: VehicleAvailability.Available,
  height: 0,
  width: 0,
  length: 0,
  weight: 0,
  capacities: { water: 0, cargo_volume: undefined },
  description: '',
}

describe('FormForVehicle', () => {
  const mockOnSubmit = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    offered = [water, cargo]
  })

  it('renders all form fields', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    expect(screen.getByLabelText(/kennzeichen/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/fahrzeugmodell/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/fahrzeugtyp/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/wassertank/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/verfügbarkeit/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/führerscheinklasse/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/höhe des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/breite des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/länge des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/gewicht des fahrzeugs/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('renders vehicle type select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const typeSelect = screen.getByRole('combobox', { name: /fahrzeugtyp/i })
    await user.click(typeSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Anhänger')).toBeInTheDocument()
    expect(within(listbox).getByText('Transporter')).toBeInTheDocument()
    expect(within(listbox).getByText('Unbekannt')).toBeInTheDocument()
  })

  it('renders driving license select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const licenseSelect = screen.getByRole('combobox', { name: /führerscheinklasse/i })
    await user.click(licenseSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('B')).toBeInTheDocument()
    expect(within(listbox).getByText('BE')).toBeInTheDocument()
    expect(within(listbox).getByText('C')).toBeInTheDocument()
    expect(within(listbox).getByText('CE')).toBeInTheDocument()
  })

  it('renders vehicle availability select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const availabilitySelect = screen.getByRole('combobox', { name: /verfügbarkeit/i })
    await user.click(availabilitySelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Verfügbar')).toBeInTheDocument()
    expect(within(listbox).getByText('Nicht Verfügbar')).toBeInTheDocument()
    expect(within(listbox).queryByText('Im Einsatz')).not.toBeInTheDocument()
  })

  it('shows error message when displayError is true', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle
          displayError={true}
          errorMessage="Ein Fehler ist aufgetreten"
          onSubmit={mockOnSubmit}
          capacities={offered}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/ein fehler ist aufgetreten/i)).toBeInTheDocument()
  })

  // Without this the user is stuck: no message tells them what is wrong, and the
  // only control that would produce one is greyed out.
  it('lets an incomplete form be submitted so its errors become visible', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const submitButton = screen.getByRole('button', { name: /speichern/i })
    expect(submitButton).not.toBeDisabled()

    await user.click(submitButton)

    expect(await screen.findAllByRole('alert')).not.toHaveLength(0)
    expect(mockOnSubmit).not.toHaveBeenCalled()
  })

  it('submits a valid form', async () => {
    const user = userEvent.setup()

    const validValues: VehicleForm = {
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

    render(
      <TestWrapper defaultValues={validValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(mockOnSubmit).toHaveBeenCalled())
  })

  it('allows entering number plate and model', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
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
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const waterCapacityInput = screen.getByLabelText(/wassertank/i)
    await user.clear(waterCapacityInput)
    await user.type(waterCapacityInput, '1500')

    expect(waterCapacityInput).toHaveValue(1500)
  })

  it('allows selecting vehicle type', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const typeSelect = screen.getByRole('combobox', { name: /fahrzeugtyp/i })
    await user.click(typeSelect)

    const listbox = await screen.findByRole('listbox')
    await user.click(within(listbox).getByText('Anhänger'))

    await waitFor(() => {
      expect(screen.getByRole('combobox', { name: /fahrzeugtyp/i })).toHaveTextContent(/anhänger/i)
    })
  })

  it('submits numeric fields as numbers, accepting comma decimals', async () => {
    const user = userEvent.setup()

    const validValues: VehicleForm = {
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

    render(
      <TestWrapper defaultValues={validValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const heightInput = screen.getByLabelText(/höhe des fahrzeugs/i)
    await user.clear(heightInput)
    await user.type(heightInput, '1,88')

    const waterCapacityInput = screen.getByLabelText(/wassertank/i)
    await user.clear(waterCapacityInput)
    await user.type(waterCapacityInput, '1500')

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).not.toBeDisabled()
    })
    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(mockOnSubmit).toHaveBeenCalled())

    const submitted = mockOnSubmit.mock.calls[0][0] as VehicleForm
    expect(submitted.height).toBe(1.88)
    expect(submitted.capacities.water).toBe(1500)
    expect(submitted.width).toBe(2)
    expect(submitted.length).toBe(6)
    expect(submitted.weight).toBe(3.5)
  })

  it('shows one field per kind of the modules active in the organization', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    const group = screen.getByRole('group', { name: /kapazität/i })
    expect(within(group).getByLabelText(/wassertank/i)).toBeInTheDocument()
    expect(within(group).getByLabelText(/laderaum/i)).toBeInTheDocument()
    expect(within(group).getAllByText('Liter')).toHaveLength(2)
    expect(within(group).getByText('für Bewässerung')).toBeInTheDocument()
    expect(within(group).getByText('für Abfall')).toBeInTheDocument()
  })

  it('shows only the kinds it is offered', () => {
    offered = [water]

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    expect(screen.getByLabelText(/wassertank/i)).toBeInTheDocument()
    expect(screen.queryByLabelText(/laderaum/i)).not.toBeInTheDocument()
  })

  it('shows no capacity section without an offered kind', async () => {
    const user = userEvent.setup()
    offered = []

    render(
      <TestWrapper
        defaultValues={{
          ...defaultFormValues,
          numberPlate: 'HH-AB-1234',
          model: 'Sprinter',
          height: 2.5,
          width: 2,
          length: 6,
          weight: 3.5,
          capacities: {},
        }}
      >
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    expect(screen.queryByRole('group', { name: /kapazität/i })).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: /speichern/i }))
    await waitFor(() => expect(mockOnSubmit).toHaveBeenCalled())
  })

  it('asks for at least one capacity when every offered kind is empty', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper
        defaultValues={{
          ...defaultFormValues,
          capacities: { water: undefined, cargo_volume: undefined, sand: 12.5 },
        }}
      >
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    await user.click(screen.getByRole('button', { name: /speichern/i }))

    const group = screen.getByRole('group', { name: /kapazität/i })
    expect(await within(group).findByRole('alert')).toHaveTextContent('vehicle.capacities.empty')
    expect(mockOnSubmit).not.toHaveBeenCalled()
  })

  it("reports the module validator's issue at the field", async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={{ ...defaultFormValues, capacities: { water: -5 } }}>
        <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
      </TestWrapper>,
    )

    await user.click(screen.getByRole('button', { name: /speichern/i }))

    const field = screen.getByLabelText(/wassertank/i)
    await waitFor(() => expect(field).toHaveAttribute('aria-invalid', 'true'))
    expect(screen.getByText('capacity.amount.outOfRange')).toBeInTheDocument()
    expect(mockOnSubmit).not.toHaveBeenCalled()
  })

  describe('untouched create form', () => {
    const DirtyProbe = () => {
      const { isDirty } = useFormState()
      return <output data-testid="dirty">{String(isDirty)}</output>
    }

    // A dirty flag here makes leaving the empty form ask "Seite verlassen?".
    it.each([
      ['only water is offered', [water]],
      ['both capacities are offered', [water, cargo]],
      ['no kind is offered', []],
    ])('is not dirty after typing and deleting when %s', async (_, kinds) => {
      offered = kinds
      const defaults = {
        ...NEW_VEHICLE_DEFAULTS,
        capacities: vehicleCapacityDefaults(kinds),
      } as VehicleForm

      render(
        <TestWrapper defaultValues={defaults}>
          <FormForVehicle displayError={false} onSubmit={mockOnSubmit} capacities={offered} />
          <DirtyProbe />
        </TestWrapper>,
      )

      // Typing and deleting again makes the form recompute its dirty flag against the defaults.
      const user = userEvent.setup()
      const numberPlate = screen.getByLabelText(/kennzeichen/i)
      await user.type(numberPlate, 'X')
      await user.clear(numberPlate)

      expect(screen.getByTestId('dirty')).toHaveTextContent('false')
    })
  })
})
