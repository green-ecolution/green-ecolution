import { describe, it, expect, vi, beforeEach } from 'vitest'
import { Suspense } from 'react'
import userEvent from '@testing-library/user-event'
import type { WasteBinResponse } from '@green-ecolution/backend-client'
import { render, screen, waitFor } from '@/test/utils'
import { planInList } from '@/test/plans'
import BinOutcomeInput from './BinOutcomeInput'

const listWasteBins = vi.fn<(params: unknown) => Promise<unknown>>()
vi.mock('../../api/client', () => ({
  wasteBinApi: { listWasteBins: (params: unknown) => listWasteBins(params) },
}))

const bins = [
  { id: 'b1', number: 'B-0001', fillState: 'full', volumeLiters: 1100 },
  { id: 'b2', number: 'B-0002', fillState: 'normal', volumeLiters: 240 },
] as WasteBinResponse[]
const plan = planInList({ module: 'waste', stops: ['b1', 'hidden', 'b2'] })

const renderInput = () => {
  const onChange = vi.fn()
  render(
    <Suspense fallback={null}>
      <BinOutcomeInput plan={plan} onChange={onChange} />
    </Suspense>,
  )
  return onChange
}

describe('BinOutcomeInput', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    listWasteBins.mockResolvedValue({
      data: bins,
      pagination: { totalPages: 1, currentPage: 1, totalRecords: bins.length },
    })
  })

  it('starts every visible bin as emptied without an amount', async () => {
    const onChange = renderInput()

    expect(await screen.findByRole('switch', { name: 'B-0001 geleert' })).toBeChecked()
    expect(screen.getByRole('switch', { name: 'B-0002 geleert' })).toBeChecked()
    await waitFor(() =>
      expect(onChange).toHaveBeenLastCalledWith(
        [
          { stop: 'b1', completed: true, amount: undefined },
          { stop: 'b2', completed: true, amount: undefined },
        ],
        true,
      ),
    )
  })

  it('reports a bin left full and an entered amount', async () => {
    const user = userEvent.setup()
    const onChange = renderInput()

    await user.click(await screen.findByRole('switch', { name: 'B-0002 geleert' }))
    await user.type(screen.getByRole('spinbutton', { name: 'Menge für B-0001 in Litern' }), '900')

    await waitFor(() =>
      expect(onChange).toHaveBeenLastCalledWith(
        [
          { stop: 'b1', completed: true, amount: 900 },
          { stop: 'b2', completed: false, amount: undefined },
        ],
        true,
      ),
    )
  })

  it('reports a negative amount as invalid', async () => {
    const user = userEvent.setup()
    const onChange = renderInput()

    await user.type(
      await screen.findByRole('spinbutton', { name: 'Menge für B-0001 in Litern' }),
      '-5',
    )

    await waitFor(() => expect(onChange).toHaveBeenLastCalledWith([], false))
  })
})
