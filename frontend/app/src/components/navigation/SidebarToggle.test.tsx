import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import SidebarToggle from './SidebarToggle'

describe('SidebarToggle', () => {
  afterEach(() => {
    cleanup()
  })

  it('has the collapse label and fires onToggle when expanded', async () => {
    const user = userEvent.setup()
    const onToggle = vi.fn()
    render(<SidebarToggle collapsed={false} onToggle={onToggle} />)

    const button = screen.getByRole('button', { name: 'Seitennavigation einklappen' })
    expect(button).toHaveAttribute('aria-expanded', 'true')
    expect(button).toHaveAttribute('aria-controls', 'main-navigation')

    await user.click(button)
    expect(onToggle).toHaveBeenCalledTimes(1)
  })

  it('has the expand label when collapsed', () => {
    render(<SidebarToggle collapsed onToggle={vi.fn()} />)

    const button = screen.getByRole('button', { name: 'Seitennavigation ausklappen' })
    expect(button).toHaveAttribute('aria-expanded', 'false')
  })
})
