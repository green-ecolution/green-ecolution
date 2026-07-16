import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import FormForTreecluster from './FormForTreecluster'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import { FormProvider, useForm } from 'react-hook-form'
import { clusterDraftResolver } from '@green-ecolution/domain-wasm'
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
    resolver: clusterDraftResolver<TreeclusterForm>(),
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
  soilCondition: SoilCondition.Unknown,
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
    expect(screen.getByRole('combobox', { name: /bodenbeschaffenheit/i })).toBeInTheDocument()
    expect(screen.getByLabelText(/kurze beschreibung/i)).toBeInTheDocument()
  })

  it('renders soil condition combobox with KA5 options', async () => {
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
    expect(within(listbox).getByText('Ss – Reinsand')).toBeInTheDocument()
    expect(within(listbox).getByText('Uu – reiner Schluff')).toBeInTheDocument()
    expect(within(listbox).getByText('Lu – schluffiger Lehm')).toBeInTheDocument()
    expect(within(listbox).getByText('Tt – reiner Ton')).toBeInTheDocument()
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
    await user.click(within(listbox).getByText('Ss – Reinsand'))

    await waitFor(() => {
      expect(screen.getByRole('combobox', { name: /bodenbeschaffenheit/i })).toHaveTextContent(
        'Ss – Reinsand',
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

    expect(screen.getByText(/noch keine bäume ausgewählt/i)).toBeInTheDocument()
  })
})

describe('soil texture dialog integration', () => {
  const mockOnSubmit = vi.fn()
  const mockOnAddTrees = vi.fn()

  it('opens the dialog via the icon button and applies the determined condition', async () => {
    render(
      <TestWrapper defaultValues={defaultFormValues}>
        <FormForTreecluster
          displayError={false}
          onAddTrees={mockOnAddTrees}
          onSubmit={mockOnSubmit}
        />
      </TestWrapper>,
    )

    await userEvent.click(screen.getByRole('button', { name: /bodenbeschaffenheit bestimmen/i }))
    expect(screen.getByRole('dialog', { name: /bodenart bestimmen/i })).toBeInTheDocument()

    // Unknown has no region → neutral 33/34/33 → classify(34, 33) = Lt2
    await userEvent.click(screen.getByRole('button', { name: /übernehmen/i }))

    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: /bodenart bestimmen/i })).not.toBeInTheDocument()
    })
    expect(
      within(screen.getByRole('combobox', { name: /bodenbeschaffenheit/i })).getByText(
        /Lt2 – schwach toniger Lehm/,
      ),
    ).toBeInTheDocument()
  })
})
