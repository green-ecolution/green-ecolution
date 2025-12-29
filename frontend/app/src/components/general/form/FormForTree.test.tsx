import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
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

const mockTreeClusters: TreeCluster[] = [
  {
    id: 1,
    name: 'Cluster A',
    address: 'Address A',
    description: '',
    soilCondition: 'good',
    treeIds: [],
    wateringStatus: 'good',
    region: { id: 1, name: 'Region 1' },
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 2,
    name: 'Cluster B',
    address: 'Address B',
    description: '',
    soilCondition: 'good',
    treeIds: [],
    wateringStatus: 'good',
    region: { id: 1, name: 'Region 1' },
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
]

const mockSensors: Sensor[] = [
  {
    id: 'sensor-1',
    status: 'online',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'sensor-2',
    status: 'offline',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
]

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

  it('populates tree cluster select with options', () => {
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

    const clusterSelect = screen.getByLabelText(/bewässerungsgruppe/i) as HTMLSelectElement

    const options = Array.from(clusterSelect.options).map((opt) => opt.text)
    expect(options).toContain('Keine Bewässerungsgruppe')
    expect(options).toContain('Cluster A')
    expect(options).toContain('Cluster B')
  })

  it('populates sensor select with options', () => {
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

    const sensorSelect = screen.getByLabelText(/verknüpfter sensor/i) as HTMLSelectElement

    const options = Array.from(sensorSelect.options).map((opt) => opt.text)
    expect(options).toContain('Kein Sensor')
    expect(options).toContain('Sensor sensor-1')
    expect(options).toContain('Sensor sensor-2')
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

    const clusterSelect = screen.getByLabelText(/bewässerungsgruppe/i)
    await user.selectOptions(clusterSelect, '1')

    expect((clusterSelect as HTMLSelectElement).value).toBe('1')

    await waitFor(() => {
      const submitButton = screen.getByRole('button', { name: /speichern/i })
      expect(submitButton).not.toBeDisabled()
    })
  })
})
