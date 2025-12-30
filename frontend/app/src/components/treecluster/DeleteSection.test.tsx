import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import DeleteSection from './DeleteSection'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import ToastProvider from '@/context/ToastContext'
import { ReactNode } from 'react'

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
}))

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false, throwOnError: false },
    },
  })

  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <ToastProvider>{children}</ToastProvider>
    </QueryClientProvider>
  )
}

describe('DeleteSection', () => {
  const mockMutationFn = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Rendering', () => {
    it('renders delete button with correct text', () => {
      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      expect(screen.getByRole('button', { name: /löschen/i })).toBeInTheDocument()
    })

    it('renders archive button when type is archive', () => {
      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="das Fahrzeug"
          type="archive"
          redirectUrl={{ to: '/vehicles' }}
        />,
        { wrapper: createWrapper() },
      )

      expect(screen.getByRole('button', { name: /archivieren/i })).toBeInTheDocument()
    })
  })

  describe('Modal Behavior', () => {
    it('opens modal when delete button is clicked', async () => {
      const user = userEvent.setup()

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))

      expect(screen.getByText(/soll der baum wirklich gelöscht werden/i)).toBeInTheDocument()
    })

    it('opens modal with archive text when type is archive', async () => {
      const user = userEvent.setup()

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="das Fahrzeug"
          type="archive"
          redirectUrl={{ to: '/vehicles' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /archivieren/i }))

      expect(screen.getByText(/soll das fahrzeug wirklich archiviert werden/i)).toBeInTheDocument()
    })

    it('shows confirmation button in modal', async () => {
      const user = userEvent.setup()

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))

      expect(screen.getByRole('button', { name: /bestätigen/i })).toBeInTheDocument()
    })

    it('closes modal when cancel is clicked', async () => {
      const user = userEvent.setup()

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      expect(screen.getByRole('dialog')).toBeInTheDocument()

      const cancelButton = screen.getByRole('button', { name: /abbrechen/i })
      await user.click(cancelButton)

      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
      })
    })
  })

  describe('Delete Mutation', () => {
    it('calls mutationFn when confirm is clicked', async () => {
      const user = userEvent.setup()
      mockMutationFn.mockResolvedValueOnce(undefined)

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      await user.click(screen.getByRole('button', { name: /bestätigen/i }))

      await waitFor(() => {
        expect(mockMutationFn).toHaveBeenCalledTimes(1)
      })
    })

    it('does not call mutationFn when cancel is clicked', async () => {
      const user = userEvent.setup()

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      await user.click(screen.getByRole('button', { name: /abbrechen/i }))

      expect(mockMutationFn).not.toHaveBeenCalled()
    })
  })

  describe('Entity Types', () => {
    it('works with Tree entity', async () => {
      const user = userEvent.setup()
      const deleteTreeFn = vi.fn().mockResolvedValue(undefined)

      render(
        <DeleteSection
          mutationFn={deleteTreeFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      expect(screen.getByText(/soll der baum wirklich gelöscht werden/i)).toBeInTheDocument()
    })

    it('works with Vehicle entity (archive)', async () => {
      const user = userEvent.setup()
      const archiveVehicleFn = vi.fn().mockResolvedValue(undefined)

      render(
        <DeleteSection
          mutationFn={archiveVehicleFn}
          entityName="das Fahrzeug"
          type="archive"
          redirectUrl={{ to: '/vehicles' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /archivieren/i }))
      expect(screen.getByText(/soll das fahrzeug wirklich archiviert werden/i)).toBeInTheDocument()
    })

    it('works with TreeCluster entity', async () => {
      const user = userEvent.setup()
      const deleteClusterFn = vi.fn().mockResolvedValue(undefined)

      render(
        <DeleteSection
          mutationFn={deleteClusterFn}
          entityName="die Bewässerungsgruppe"
          redirectUrl={{ to: '/treecluster' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      expect(
        screen.getByText(/soll die bewässerungsgruppe wirklich gelöscht werden/i),
      ).toBeInTheDocument()
    })

    it('works with WateringPlan entity', async () => {
      const user = userEvent.setup()
      const deletePlanFn = vi.fn().mockResolvedValue(undefined)

      render(
        <DeleteSection
          mutationFn={deletePlanFn}
          entityName="der Einsatzplan"
          redirectUrl={{ to: '/watering-plans' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      expect(screen.getByText(/soll der einsatzplan wirklich gelöscht werden/i)).toBeInTheDocument()
    })
  })

  describe('Error Handling', () => {
    it('handles mutation error gracefully', async () => {
      const user = userEvent.setup()
      const error = new Error('Delete failed')
      mockMutationFn.mockRejectedValueOnce(error)

      // eslint-disable-next-line @typescript-eslint/no-empty-function
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      render(
        <DeleteSection
          mutationFn={mockMutationFn}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />,
        { wrapper: createWrapper() },
      )

      await user.click(screen.getByRole('button', { name: /löschen/i }))
      await user.click(screen.getByRole('button', { name: /bestätigen/i }))

      await waitFor(() => {
        expect(mockMutationFn).toHaveBeenCalledTimes(1)
      })

      consoleSpy.mockRestore()
    })
  })
})
