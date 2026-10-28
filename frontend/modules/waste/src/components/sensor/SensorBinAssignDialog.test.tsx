import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { SensorResponse } from '@green-ecolution/backend-client'
import { wasteBinApi } from '../../api/client'
import { wasteBinFixtures } from '../../test/handlers'
import SensorBinAssignDialog from './SensorBinAssignDialog'

const sensor = { id: 'eui-1', target: null, location: null } as unknown as SensorResponse

const renderDialog = (onConfirm = vi.fn(), error: string | null = null) =>
  render(
    <SensorBinAssignDialog
      open
      onOpenChange={vi.fn()}
      mode="activate"
      sensor={sensor}
      pending={false}
      error={error}
      onConfirm={onConfirm}
    />,
  )

describe('SensorBinAssignDialog', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.spyOn(wasteBinApi, 'listWasteBins').mockResolvedValue({
      data: wasteBinFixtures,
      pagination: { currentPage: 1, perPage: 100, totalPages: 1, totalRecords: 2 },
    })
  })

  it('confirms the picked bin as a selected target', async () => {
    const onConfirm = vi.fn()
    renderDialog(onConfirm)

    expect(screen.getByRole('button', { name: 'Aktivieren' })).toBeDisabled()
    await userEvent.click(await screen.findByText('B-0043'))
    await userEvent.click(screen.getByRole('button', { name: 'Aktivieren' }))

    expect(onConfirm).toHaveBeenCalledExactlyOnceWith({
      target: { module: 'waste', kind: 'bin', id: wasteBinFixtures[1].id },
      label: 'B-0043',
    })
  })

  it('narrows the list by bin number', async () => {
    renderDialog()
    await screen.findByText('B-0043')

    await userEvent.type(screen.getByRole('searchbox'), '0043')

    expect(screen.queryByText('B-0042')).not.toBeInTheDocument()
    expect(screen.getByText('B-0043')).toBeInTheDocument()
  })

  it('does not offer a bin that already has a sensor', async () => {
    renderDialog()

    expect(await screen.findByRole('button', { name: /B-0042/ })).toBeDisabled()
    expect(screen.getByRole('button', { name: /B-0043/ })).toBeEnabled()
  })

  it('shows the error the core reports back', async () => {
    renderDialog(vi.fn(), 'Der Behälter hat bereits einen Sensor.')
    expect(await screen.findByText('Der Behälter hat bereits einen Sensor.')).toBeInTheDocument()
  })
})
