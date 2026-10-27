import { describe, it, expect, vi, beforeEach } from 'vitest'
import { Suspense } from 'react'
import userEvent from '@testing-library/user-event'
import type { TreeClusterInListResponse } from '@green-ecolution/backend-client'
import { render, screen, waitFor } from '@/test/utils'
import { planInList } from '@/test/plans'
import ClusterOutcomeInput from './ClusterOutcomeInput'

const loadStopClusters = vi.fn<(stops: string[]) => Promise<TreeClusterInListResponse[]>>()
vi.mock('../../api/plans', () => ({
  loadStopClusters: (stops: string[]) => loadStopClusters(stops),
}))

const cluster = {
  id: 'c1',
  name: 'Am Hafen',
  treeIds: ['t1', 't2', 't3'],
} as TreeClusterInListResponse
const plan = planInList({ stops: ['c1', 'hidden'] })

const renderInput = () => {
  const onChange = vi.fn()
  render(
    <Suspense fallback={null}>
      <ClusterOutcomeInput plan={plan} onChange={onChange} />
    </Suspense>,
  )
  return onChange
}

describe('ClusterOutcomeInput', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    loadStopClusters.mockResolvedValue([cluster])
  })

  it('starts each visible cluster at 80 liters per tree and reports it as valid', async () => {
    const onChange = renderInput()

    expect(await screen.findByRole('spinbutton', { name: 'Liter für Am Hafen' })).toHaveValue(240)
    await waitFor(() =>
      expect(onChange).toHaveBeenLastCalledWith(
        [{ stop: 'c1', completed: true, amount: 240 }],
        true,
      ),
    )
  })

  it('reports the edited amount', async () => {
    const user = userEvent.setup()
    const onChange = renderInput()

    const input = await screen.findByRole('spinbutton', { name: 'Liter für Am Hafen' })
    await user.clear(input)
    await user.type(input, '500')

    await waitFor(() =>
      expect(onChange).toHaveBeenLastCalledWith(
        [{ stop: 'c1', completed: true, amount: 500 }],
        true,
      ),
    )
  })

  it('reports an empty amount as invalid', async () => {
    const user = userEvent.setup()
    const onChange = renderInput()

    await user.clear(await screen.findByRole('spinbutton', { name: 'Liter für Am Hafen' }))

    await waitFor(() => expect(onChange).toHaveBeenLastCalledWith([], false))
  })
})
