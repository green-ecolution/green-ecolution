import { useState } from 'react'
import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'

const params = vi.hoisted(() => ({ orgId: 'amt' }))

vi.mock('@tanstack/react-router', () => {
  const Counter = () => {
    const [count, setCount] = useState(0)
    return <button onClick={() => setCount(count + 1)}>clicks {count}</button>
  }
  return {
    getRouteApi: () => ({ useParams: () => params }),
    Outlet: Counter,
  }
})

const { default: SettingsOrganizationOutlet } = await import('./SettingsOrganizationOutlet')

describe('SettingsOrganizationOutlet', () => {
  it('keeps the page state while the organization stays the same', async () => {
    params.orgId = 'amt'
    const { rerender } = render(<SettingsOrganizationOutlet />)
    await userEvent.click(screen.getByRole('button'))

    rerender(<SettingsOrganizationOutlet />)

    expect(screen.getByRole('button')).toHaveTextContent('clicks 1')
  })

  it('remounts the page when the organization changes', async () => {
    params.orgId = 'amt'
    const { rerender } = render(<SettingsOrganizationOutlet />)
    await userEvent.click(screen.getByRole('button'))

    params.orgId = 'nord'
    rerender(<SettingsOrganizationOutlet />)

    expect(screen.getByRole('button')).toHaveTextContent('clicks 0')
  })
})
