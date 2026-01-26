import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForTreecluster from './FormForTreecluster'
import { clusterSchema, TreeclusterForm } from '@/schema/treeclusterSchema'
import { FormProvider, useForm } from 'react-hook-form'
import { zodResolver } from '@/lib/zodResolver'
import { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Toaster } from '@green-ecolution/ui'
import { SoilCondition } from '@green-ecolution/backend-client'

function TestWrapper({
  children,
  defaultValues,
}: {
  children: ReactNode
  defaultValues: TreeclusterForm
}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  const methods = useForm<TreeclusterForm>({
    defaultValues,
    resolver: zodResolver(clusterSchema),
    mode: 'onChange',
  })

  return (
    <QueryClientProvider client={queryClient}>
      <FormProvider {...methods}>{children}</FormProvider>
      <Toaster />
    </QueryClientProvider>
  )
}

const defaultFormValues: TreeclusterForm = {
  name: '',
  address: '',
  description: '',
  soilCondition: SoilCondition.TreeSoilConditionUnknown,
  treeIds: [],
}

describe('FormForTreecluster', () => {
  const mockOnSubmit = vi.fn()
  const mockOnAddTrees = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders all form fields', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByLabelText(/name/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/adresse/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/bodenbeschaffenheit/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('renders soil condition select with options', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const soilSelect = screen.getByRole('combobox', { name: /bodenbeschaffenheit/i })
    await user.click(soilSelect)

    const listbox = await screen.findByRole('listbox')
    expect(within(listbox).getByText('Schluffig')).toBeInTheDocument()
    expect(within(listbox).getByText('Sandig')).toBeInTheDocument()
    expect(within(listbox).getByText('Lehmig')).toBeInTheDocument()
    expect(within(listbox).getByText('Tonig')).toBeInTheDocument()
    expect(within(listbox).getByText('Unbekannt')).toBeInTheDocument()
  })

  it('renders add trees button', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByRole('button', { name: /bäume hinzufügen/i })).toBeInTheDocument()
  })

  it('calls onAddTrees when add trees button is clicked', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const addButton = screen.getByRole('button', { name: /bäume hinzufügen/i })
    await user.click(addButton)

    expect(mockOnAddTrees).toHaveBeenCalled()
  })

  it('shows error message when displayError is true', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={true}
          errorMessage="Ein Fehler ist aufgetreten"
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/ein fehler ist aufgetreten/i)).toBeInTheDocument()
  })

  it('submit button is disabled when form is invalid', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const submitButton = screen.getByRole('button', { name: /speichern/i })
    expect(submitButton).toBeDisabled()
  })

  it('submit button is enabled when form is valid', async () => {
    const user = userEvent.setup()

    const validValues: TreeclusterForm = {
      ...defaultFormValues,
      name: 'Test Cluster',
      address: 'Test Address',
    }

    render(
      <TestWrapper defaultValues={validValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const nameInput = screen.getByLabelText(/name/i)
    await user.click(nameInput)
    await user.tab()

    await waitFor(() => {
      const submitButton = screen.getByRole('button', { name: /speichern/i })
      expect(submitButton).not.toBeDisabled()
    })
  })

  it('allows entering name and address', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const nameInput = screen.getByLabelText(/name/i)
    const addressInput = screen.getByLabelText(/adresse/i)

    await user.type(nameInput, 'New Cluster')
    await user.type(addressInput, 'New Address 123')

    expect(nameInput).toHaveValue('New Cluster')
    expect(addressInput).toHaveValue('New Address 123')
  })

  it('allows selecting soil condition', async () => {
    const user = userEvent.setup()

    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    const soilSelect = screen.getByRole('combobox', { name: /bodenbeschaffenheit/i })
    await user.click(soilSelect)

    const listbox = await screen.findByRole('listbox')
    await user.click(within(listbox).getByText('Sandig'))

    await waitFor(() => {
      expect(screen.getByRole('combobox', { name: /bodenbeschaffenheit/i })).toHaveTextContent(
        /sandig/i,
      )
    })
  })

  it('displays empty trees message when no trees selected', () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    expect(screen.getByText(/hier können sie zugehörige bäume verlinken/i)).toBeInTheDocument()
  })
})
