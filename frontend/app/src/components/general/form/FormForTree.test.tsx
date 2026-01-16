import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForTree from './FormForTree'
import { treeSchema, TreeForm } from '@/schema/treeSchema'
import { FormProvider, useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import ToastProvider from '@/context/ToastContext'
import { TreeCluster, Sensor } from '@green-ecolution/backend-client'

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

  const methods = useForm<TreeForm>({
    defaultValues,
    resolver: zodResolver(treeSchema),
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

const defaultFormValues: TreeForm = {
  latitude: 53.5511,
  longitude: 9.9937,
  number: '',
  species: '',
  plantingYear: 2024,
  treeClusterId: -1,
  sensorId: '-1',
  description: '',
}

const mockTreeClusters = [
  { id: 1, name: 'Cluster A' },
  { id: 2, name: 'Cluster B' },
] as TreeCluster[]

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

  it('submit button is disabled when form is invalid', () => {
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
    expect(submitButton).toBeDisabled()
  })

  it('submit button is enabled when form is valid', async () => {
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

    // Trigger validation by interacting with the form
    const numberInput = screen.getByLabelText(/baumnummer/i)
    await user.click(numberInput)
    await user.tab()

    await waitFor(() => {
      const submitButton = screen.getByRole('button', { name: /speichern/i })
      expect(submitButton).not.toBeDisabled()
    })
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

    await waitFor(() => {
      const submitButton = screen.getByRole('button', { name: /speichern/i })
      expect(submitButton).not.toBeDisabled()
    })
  })
})
