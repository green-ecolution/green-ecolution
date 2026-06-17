import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, fireEvent } from '@/test/utils'
import ClusterPanelShell from './ClusterPanelShell'

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

  it('closes on Escape', () => {
    const onClose = vi.fn()
    render(
      <ClusterPanelShell onClose={onClose}>
        <p>x</p>
      </ClusterPanelShell>,
    )
    fireEvent.keyDown(document, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledTimes(1)
  })
})
