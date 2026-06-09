import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, userEvent } from '@/test/utils'
import { WateringStatus } from '@green-ecolution/backend-client'
import MapFilterToolbar from './MapFilterToolbar'

afterEach(cleanup)

const renderToolbar = (over: Partial<React.ComponentProps<typeof MapFilterToolbar>> = {}) =>
  render(
    <MapFilterToolbar
      searchTerm=""
      onSearchTermChange={vi.fn()}
      statuses={[]}
      onToggleStatus={vi.fn()}
      filterSlot={<button>Filter</button>}
      createSlot={<button>Gruppe anlegen</button>}
      {...over}
    />,
  )

describe('MapFilterToolbar', () => {
  it('reports search input changes', async () => {
    const onSearchTermChange = vi.fn()
    renderToolbar({ onSearchTermChange })
    await userEvent.type(screen.getByPlaceholderText(/Baumgruppe/), 'Haf')
    expect(onSearchTermChange).toHaveBeenCalled()
  })

  it('toggles a status chip', async () => {
    const onToggleStatus = vi.fn()
    renderToolbar({ onToggleStatus })
    await userEvent.click(screen.getByRole('button', { name: 'Sehr trocken' }))
    expect(onToggleStatus).toHaveBeenCalledWith(WateringStatus.Bad)
  })

  it('renders the filter and create slots', () => {
    renderToolbar()
    expect(screen.getByRole('button', { name: 'Filter' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Gruppe anlegen' })).toBeInTheDocument()
  })
})
