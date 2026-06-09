import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { render, screen, cleanup, userEvent } from '@/test/utils'
import ClusterPanelShell from './ClusterPanelShell'

const enableDragging = vi.fn()
const disableDragging = vi.fn()
vi.mock('@/hooks/useMapInteractions', () => ({
  default: () => ({ enableDragging, disableDragging }),
}))
vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => true }))

beforeEach(() => vi.clearAllMocks())
afterEach(cleanup)

describe('ClusterPanelShell', () => {
  it('renders children', () => {
    render(
      <ClusterPanelShell onClose={vi.fn()}>
        <p>Panel-Inhalt</p>
      </ClusterPanelShell>,
    )
    expect(screen.getByText('Panel-Inhalt')).toBeInTheDocument()
  })

  it('closes via the close button', async () => {
    const onClose = vi.fn()
    render(
      <ClusterPanelShell onClose={onClose}>
        <p>x</p>
      </ClusterPanelShell>,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Seitenansicht schließen' }))
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('disables map dragging while mounted and re-enables on unmount', () => {
    const { unmount } = render(
      <ClusterPanelShell onClose={vi.fn()}>
        <p>x</p>
      </ClusterPanelShell>,
    )
    expect(disableDragging).toHaveBeenCalled()
    unmount()
    expect(enableDragging).toHaveBeenCalled()
  })
})
