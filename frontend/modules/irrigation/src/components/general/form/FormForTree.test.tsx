import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForTree from './FormForTree'
import { TreeForm } from '../../../schema/treeSchema'
import { FormProvider, useForm } from 'react-hook-form'
import {
  irrigationPlantingYearMax,
  irrigationPlantingYearMin,
  treeDraftResolver,
} from '@green-ecolution/domain-wasm'
import { ReactNode } from 'react'
import { FORM_VALIDATION_MODE } from '@/hooks/form/useEntityForm'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Toaster } from '@green-ecolution/ui'
import { Sensor } from '@/api/backendApi'
import { TreeClusterInList } from '../../../api/client'

function TestWrapper({
  children,
  defaultValues,
}: {
  children: ReactNode
  defaultValues: TreeForm
}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  const translate = useIssueTranslator()

  // Mirrors the production wiring in useEntityForm; a divergent mode here would
  // hide exactly the bugs these tests exist to catch.
  const methods = useForm<TreeForm>({
    defaultValues,
    // The real translator, like useTreeForm uses: a stub returning the key
    // would hide whether the catalog actually resolves and interpolates.
    resolver: treeDraftResolver<TreeForm>(translate),
    mode: FORM_VALIDATION_MODE,
  })

  return (
    <QueryClientProvider client={queryClient}>
      <FormProvider {...methods}>{children}</FormProvider>
      <Toaster />
    </QueryClientProvider>
  )
}

const defaultFormValues: TreeForm = {
  latitude: 53.5511,
  longitude: 9.9937,
  number: '',
  species: '',
  plantingYear: 2024,
  treeClusterId: null,
  sensorId: null,
  description: '',
}

const mockTreeClusters = [
  { id: 'cluster-uuid-1', name: 'Cluster A' },
  { id: 'cluster-uuid-2', name: 'Cluster B' },
] as unknown as TreeClusterInList[]

const mockSensors = [
  { id: 'sensor-1', status: 'online' },
  { id: 'sensor-2', status: 'offline' },
] as Sensor[]

describe('FormForTree', () => {
  const mockOnSubmit = vi.fn()
  const mockOnChangeLocation = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders all form fields when not readonly', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByLabelText(/baumnummer/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/baumart/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/pflanzjahr/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/bewässerungsgruppe/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/verknüpfter sensor/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('hides editable fields when readonly is true', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={true}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.queryByLabelText(/baumnummer/i)).not.toBeInTheDocument()
    expect(screen.queryByLabelText(/baumart/i)).not.toBeInTheDocument()
    expect(screen.queryByLabelText(/pflanzjahr/i)).not.toBeInTheDocument()
    expect(screen.queryByLabelText(/bewässerungsgruppe/i)).not.toBeInTheDocument()
    // Sensor and description should still be visible
    expect(screen.getByLabelText(/verknüpfter sensor/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('populates tree cluster select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const clusterSelect = screen.getByRole('combobox', { name: /bewässerungsgruppe/i })
    await user.click(clusterSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Keine Bewässerungsgruppe')).toBeInTheDocument()
    expect(within(listbox).getByText('Cluster A')).toBeInTheDocument()
    expect(within(listbox).getByText('Cluster B')).toBeInTheDocument()
  })

  it('populates sensor select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const sensorSelect = screen.getByRole('combobox', { name: /verknüpfter sensor/i })
    await user.click(sensorSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Kein Sensor')).toBeInTheDocument()
    expect(within(listbox).getByText('Sensor sensor-1')).toBeInTheDocument()
    expect(within(listbox).getByText('Sensor sensor-2')).toBeInTheDocument()
  })

  it('displays coordinates', () => {
    const valuesWithCoords = {
      ...defaultFormValues,
      latitude: 53.5511,
      longitude: 9.9937,
    }

    render(
      <TestWrapper defaultValues={valuesWithCoords}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/53\.5511/)).toBeInTheDocument()
    expect(screen.getByText(/9\.9937/)).toBeInTheDocument()
  })

  it('calls onChangeLocation when location button is clicked', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const locationButton = screen.getByRole('button', {
      name: /standort des baumes anpassen/i,
    })
    await user.click(locationButton)

    expect(mockOnChangeLocation).toHaveBeenCalled()
  })

  it('shows error message when displayError is true', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={true}
          errorMessage="Ein Fehler ist aufgetreten"
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/ein fehler ist aufgetreten/i)).toBeInTheDocument()
  })

  it('submits a valid form', async () => {
    const user = userEvent.setup()

    const validValues = {
      ...defaultFormValues,
      number: 'T-001',
      species: 'Oak',
    }

    render(
      <TestWrapper defaultValues={validValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(mockOnSubmit).toHaveBeenCalled())
  })

  it('form remains valid after selecting a tree cluster (#513)', async () => {
    const user = userEvent.setup()

    const validValues = {
      ...defaultFormValues,
      number: 'T-001',
      species: 'Oak',
    }

    render(
      <TestWrapper defaultValues={validValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    await waitFor(() => {
      const submitButton = screen.getByRole('button', { name: /speichern/i })
      expect(submitButton).not.toBeDisabled()
    })

    const clusterSelect = screen.getByRole('combobox', { name: /bewässerungsgruppe/i })
    await user.click(clusterSelect)

    const listbox = await screen.findByRole('listbox')
    await user.click(within(listbox).getByText('Cluster A'))

    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(mockOnSubmit).toHaveBeenCalled())
  })

  it('expands a shorthand planting year when the field is left (#61)', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    await user.clear(plantingYear)
    await user.type(plantingYear, '25')
    await user.tab()

    await waitFor(() => expect(plantingYear).toHaveValue(2025))
  })

  // Without bounds the spinner arrows walk straight past what the domain accepts.
  it('constrains the planting year stepper to the supported range', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    expect(plantingYear).toHaveAttribute('min', String(irrigationPlantingYearMin()))
    expect(plantingYear).toHaveAttribute('max', String(irrigationPlantingYearMax()))
  })

  // A future year is valid input (a planned planting), so this is a hint and
  // not one of the red validation errors.
  it('points out a planting year that lies in the future', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    await user.clear(plantingYear)
    await user.type(plantingYear, String(new Date().getFullYear() + 3))

    expect(await screen.findByText(/liegt in der zukunft/i)).toBeInTheDocument()
  })

  it('keeps quiet about the future for a planting year that has passed', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    await user.clear(plantingYear)
    await user.type(plantingYear, '2020')

    expect(screen.queryByText(/liegt in der zukunft/i)).not.toBeInTheDocument()
  })

  it('blocks the sensor selection while the planting year lies in the future', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const sensorSelect = screen.getByRole('combobox', { name: /verknüpfter sensor/i })
    expect(sensorSelect).not.toBeDisabled()

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    await user.clear(plantingYear)
    await user.type(plantingYear, String(new Date().getFullYear() + 3))

    await waitFor(() => expect(sensorSelect).toBeDisabled())
    expect(screen.getByText(/kann kein sensor verknüpft werden/i)).toBeInTheDocument()
  })

  it('shows the validation message for an out-of-range planting year', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    await user.clear(plantingYear)
    await user.type(plantingYear, '202')
    await user.tab()

    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent(String(irrigationPlantingYearMin()))
    expect(alert).toHaveTextContent(String(irrigationPlantingYearMax()))
    expect(alert.textContent).not.toContain('{')
  })

  // Without this the user is stuck: no message tells them what is wrong, and the
  // only control that would produce one is greyed out.
  it('lets an incomplete form be submitted so its errors become visible', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const submitButton = screen.getByRole('button', { name: /speichern/i })
    expect(submitButton).not.toBeDisabled()

    await user.click(submitButton)

    expect(await screen.findAllByRole('alert')).not.toHaveLength(0)
    expect(mockOnSubmit).not.toHaveBeenCalled()
  })

  it('still reports blur to the form while expanding the planting year', async () => {
    const user = userEvent.setup()
    const onBlur = vi.fn()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTree
          isReadonly={false}
          treeClusters={mockTreeClusters}
          sensors={mockSensors}
          displayError={false}
          onChangeLocation={mockOnChangeLocation}
          onSubmit={mockOnSubmit}
          onBlur={onBlur}
        />
      </TestWrapper>,
    )

    const plantingYear = screen.getByLabelText(/pflanzjahr/i)
    await user.clear(plantingYear)
    await user.type(plantingYear, '25')
    await user.tab()

    await waitFor(() => expect(onBlur).toHaveBeenCalled())
  })
})
