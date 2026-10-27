import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import SensorTreePickerSheet from './SensorTreePickerSheet'

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('SensorTreePickerSheet', () => {
  it('does not render the search input when closed', () => {
    renderWithClient(
      <SensorTreePickerSheet
        open={false}
        onOpenChange={vi.fn()}
        selectedTreeId={null}
        onSelect={vi.fn()}
      />,
    )
    expect(screen.queryByRole('searchbox')).not.toBeInTheDocument()
  })

  it('renders title, description, search input and idle hint when open', () => {
    renderWithClient(
      <SensorTreePickerSheet
        open
        onOpenChange={vi.fn()}
        selectedTreeId={null}
        onSelect={vi.fn()}
      />,
    )
    expect(screen.getByRole('searchbox')).toBeInTheDocument()
    expect(screen.getByText(/Anderen Baum auswählen/i)).toBeInTheDocument()
    expect(screen.getByText(/Tippe Baumnummer oder Baumart/i)).toBeInTheDocument()
  })
})
