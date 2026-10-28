import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import userEvent from '@testing-library/user-event'
import type { WasteBinResponse } from '@green-ecolution/backend-client'
import { render, screen } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import useStore from '@/store/store'
import SuggestionsColumn from './SuggestionsColumn'

const permissions = vi.fn((): Permissions => new Set<string>())
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const navigate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigate,
  Link: ({ children }: { children: ReactNode }) => <a href="/">{children}</a>,
}))

const listWasteBins = vi.fn<(params: unknown) => Promise<unknown>>()
vi.mock('../../api/client', () => ({
  wasteBinApi: { listWasteBins: (params: unknown) => listWasteBins(params) },
}))

const fullBin = {
  id: 'b1',
  number: 'B-0001',
  fillState: 'full',
  fillPercent: 92,
  volumeLiters: 1100,
} as WasteBinResponse

describe('waste SuggestionsColumn', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useStore.getState().clearAllFormDrafts()
    permissions.mockReturnValue(new Set<string>())
    listWasteBins.mockResolvedValue({
      data: [fullBin],
      pagination: { totalPages: 1, currentPage: 1, totalRecords: 1 },
    })
  })

  it('suggests the bins whose own fill state is full, across every page', async () => {
    const normalBin = {
      ...fullBin,
      id: 'b2',
      number: 'B-0002',
      fillState: 'normal',
      fillPercent: 40,
    }
    const laterFullBin = { ...fullBin, id: 'b3', number: 'B-0003', fillPercent: 81 }
    listWasteBins.mockImplementation((params) =>
      Promise.resolve(
        (params as { page: number }).page === 1
          ? { data: [fullBin, normalBin], pagination: { totalPages: 2 } }
          : { data: [laterFullBin], pagination: { totalPages: 2 } },
      ),
    )
    render(<SuggestionsColumn />)

    expect(await screen.findByText('B-0001')).toBeInTheDocument()
    expect(await screen.findByText('B-0003')).toBeInTheDocument()
    expect(screen.queryByText('B-0002')).not.toBeInTheDocument()
    expect(screen.getByText('Voll · 92 %')).toBeInTheDocument()
    const header = screen.getByRole('heading', { name: 'Volle Behälter' }).parentElement
    expect(header).toHaveTextContent('2')
    expect(listWasteBins).not.toHaveBeenCalledWith(expect.objectContaining({ fillState: 'full' }))
  })

  it('offers no selection without collection_plan:create', async () => {
    permissions.mockReturnValue(new Set(['collection_plan:read', 'waste_bin:read']))
    render(<SuggestionsColumn />)

    expect(await screen.findByText('B-0001')).toBeInTheDocument()
    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /leerungstour/i })).not.toBeInTheDocument()
  })

  it('starts a collection tour with the selected bins', async () => {
    const user = userEvent.setup()
    permissions.mockReturnValue(new Set(['collection_plan:create']))
    render(<SuggestionsColumn />)

    const bundle = await screen.findByRole('button', { name: /zu leerungstour bündeln/i })
    expect(bundle).toBeDisabled()

    await user.click(screen.getByRole('checkbox', { name: 'B-0001 für Leerungstour auswählen' }))
    await user.click(screen.getByRole('button', { name: /zu leerungstour bündeln/i }))

    expect(useStore.getState().formDrafts['wateringplan-create']?.data).toMatchObject({
      module: 'waste',
      stopIds: ['b1'],
    })
    expect(navigate).toHaveBeenCalledWith({ to: '/plans/new', search: { module: 'waste' } })
  })
})
