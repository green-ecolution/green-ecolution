import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, cleanup, userEvent } from '@/test/utils'
import type { TreeClusterResponse } from '@/api/backendApi'

const onSubmit = vi.fn()
vi.mock('./useClusterPanelEdit', async () => {
  const { useForm } = await vi.importActual<typeof import('react-hook-form')>('react-hook-form')
  return {
    useClusterPanelEdit: (cluster: TreeClusterResponse) => {
      const form = useForm({
        defaultValues: {
          name: cluster.name,
          address: cluster.address,
          description: cluster.description,
          soilCondition: cluster.soilCondition,
          treeIds: [],
        },
      })
      return { form, onSubmit, isPending: false, isError: false }
    },
  }
})

import ClusterPanelEdit from './ClusterPanelEdit'

const cluster = {
  id: 'c1',
  name: 'Hafenspitze',
  address: 'Schiffbrücke 12',
  description: '',
  soilCondition: 'sandig',
  trees: [],
} as unknown as TreeClusterResponse

beforeEach(() => vi.clearAllMocks())
afterEach(cleanup)

describe('ClusterPanelEdit', () => {
  it('renders the name field with the initial value', () => {
    render(<ClusterPanelEdit treecluster={cluster} onCancel={vi.fn()} onSaved={vi.fn()} />)
    expect(screen.getByLabelText(/Name/)).toHaveValue('Hafenspitze')
  })

  it('submits on save', async () => {
    render(<ClusterPanelEdit treecluster={cluster} onCancel={vi.fn()} onSaved={vi.fn()} />)
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(onSubmit).toHaveBeenCalled()
  })

  it('cancels', async () => {
    const onCancel = vi.fn()
    render(<ClusterPanelEdit treecluster={cluster} onCancel={onCancel} onSaved={vi.fn()} />)
    await userEvent.click(screen.getByRole('button', { name: 'Abbrechen' }))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })
})
