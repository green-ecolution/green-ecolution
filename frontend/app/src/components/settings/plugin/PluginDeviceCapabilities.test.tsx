import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { PluginDeviceCapabilities } from './PluginDeviceCapabilities'

describe('PluginDeviceCapabilities', () => {
  it('shows one checkbox per capability inside a named group', () => {
    render(<PluginDeviceCapabilities capabilities={new Set(['camera'])} onToggle={vi.fn()} />)

    const group = screen.getByRole('group', { name: /gerätezugriff der ansicht/i })
    expect(group).toBeVisible()
    expect(screen.getByRole('checkbox', { name: /kamera/i })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: /bluetooth/i })).not.toBeChecked()
  })

  it('reports the toggled capability', async () => {
    const user = userEvent.setup()
    const onToggle = vi.fn()
    render(<PluginDeviceCapabilities capabilities={new Set()} onToggle={onToggle} />)

    await user.click(screen.getByRole('checkbox', { name: /bluetooth/i }))

    expect(onToggle).toHaveBeenCalledWith('bluetooth')
  })
})
