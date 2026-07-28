import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Switch } from './switch'

afterEach(cleanup)

describe('Switch', () => {
  it('exposes the checked state to assistive technology', () => {
    render(<Switch checked aria-label="Bäume ansehen" onCheckedChange={() => {}} />)
    expect(screen.getByRole('switch', { name: 'Bäume ansehen' })).toBeChecked()
  })

  it('reports the new state on click', async () => {
    const onCheckedChange = vi.fn()
    render(<Switch checked={false} aria-label="Baum anlegen" onCheckedChange={onCheckedChange} />)
    await userEvent.click(screen.getByRole('switch', { name: 'Baum anlegen' }))
    expect(onCheckedChange).toHaveBeenCalledWith(true)
  })

  it('does not report changes while disabled', async () => {
    const onCheckedChange = vi.fn()
    render(
      <Switch checked={false} disabled aria-label="Baum löschen" onCheckedChange={onCheckedChange} />,
    )
    await userEvent.click(screen.getByRole('switch', { name: 'Baum löschen' }))
    expect(onCheckedChange).not.toHaveBeenCalled()
  })
})
