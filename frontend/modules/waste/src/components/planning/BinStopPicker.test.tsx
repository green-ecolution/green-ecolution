import { describe, it, expect, vi, beforeEach } from 'vitest'
import userEvent from '@testing-library/user-event'
import type { WasteBinResponse } from '@green-ecolution/backend-client'
import { render, screen, within } from '@/test/utils'
import type { StopPickerProps } from '@/modules/types'
import BinStopPicker from './BinStopPicker'

const listWasteBins = vi.fn<(params: unknown) => Promise<unknown>>()
vi.mock('../../api/client', () => ({
  wasteBinApi: { listWasteBins: (params: unknown) => listWasteBins(params) },
}))

const bin = (id: string, number: string, fillState: WasteBinResponse['fillState']) =>
  ({ id, number, fillState, fillPercent: null, volumeLiters: 1100 }) as WasteBinResponse

const bins = [
  bin('b1', 'B-0001', 'full'),
  bin('b2', 'B-0002', 'normal'),
  bin('b3', 'B-0003', 'full'),
]

const props = (overrides: Partial<StopPickerProps> = {}): StopPickerProps => ({
  value: ['b2', 'hidden'],
  onChange: vi.fn(),
  organizationId: undefined,
  error: undefined,
  form: {
    type: 'create',
    planId: undefined,
    transporterId: undefined,
    trailerId: undefined,
    saveDraft: vi.fn(),
  },
  ...overrides,
})

const openDialog = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.click(await screen.findByRole('button', { name: 'Behälter auswählen' }))
  return screen.findByRole('dialog')
}

describe('BinStopPicker', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    listWasteBins.mockResolvedValue({
      data: bins,
      pagination: { totalPages: 1, currentPage: 1, totalRecords: bins.length },
    })
  })

  it('lists the selected bins the reader can see', async () => {
    render(<BinStopPicker {...props()} />)

    expect(await screen.findByText('B-0002')).toBeInTheDocument()
    expect(screen.queryByText('hidden')).not.toBeInTheDocument()
  })

  it('shows every bin with its fill state in the dialog', async () => {
    const user = userEvent.setup()
    render(<BinStopPicker {...props()} />)

    const dialog = await openDialog(user)

    expect(within(dialog).getByRole('checkbox', { name: /B-0001/ })).not.toBeChecked()
    expect(within(dialog).getByRole('checkbox', { name: /B-0002/ })).toBeChecked()
    expect(within(dialog).getAllByText('Voll')).toHaveLength(2)
    expect(within(dialog).getByText('Normal')).toBeInTheDocument()
  })

  it('adds exactly the full bins and keeps the stops it cannot show', async () => {
    const user = userEvent.setup()
    const pickerProps = props()
    render(<BinStopPicker {...pickerProps} />)

    const dialog = await openDialog(user)
    await user.click(within(dialog).getByRole('button', { name: 'Alle vollen auswählen' }))

    expect(pickerProps.onChange).toHaveBeenLastCalledWith(['b2', 'b1', 'b3', 'hidden'])
  })

  it('removes a bin when it is deselected', async () => {
    const user = userEvent.setup()
    const pickerProps = props()
    render(<BinStopPicker {...pickerProps} />)

    const dialog = await openDialog(user)
    await user.click(within(dialog).getByRole('checkbox', { name: /B-0002/ }))

    expect(pickerProps.onChange).toHaveBeenLastCalledWith(['hidden'])
  })

  it('adds a bin when it is selected', async () => {
    const user = userEvent.setup()
    const pickerProps = props()
    render(<BinStopPicker {...pickerProps} />)

    const dialog = await openDialog(user)
    await user.click(within(dialog).getByRole('checkbox', { name: /B-0003/ }))

    expect(pickerProps.onChange).toHaveBeenLastCalledWith(['b2', 'b3', 'hidden'])
  })

  it('removes a selected bin from the list outside the dialog', async () => {
    const user = userEvent.setup()
    const pickerProps = props()
    render(<BinStopPicker {...pickerProps} />)

    await user.click(await screen.findByRole('button', { name: 'B-0002 entfernen' }))

    expect(pickerProps.onChange).toHaveBeenLastCalledWith(['hidden'])
  })

  it('names the missing bin in its own words', () => {
    render(<BinStopPicker {...props({ value: [], error: 'Wähle mindestens einen Stopp aus.' })} />)

    expect(screen.getByRole('alert')).toHaveTextContent('Wähle mindestens einen Behälter aus.')
  })
})
