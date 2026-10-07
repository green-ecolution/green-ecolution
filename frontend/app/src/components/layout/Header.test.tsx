import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => false }))
vi.mock('@/hooks/useSidebarCollapsed', () => ({ useSidebarCollapsed: () => false }))
vi.mock('@/store/store', () => ({
  default: (selector: (s: { setSidebarCollapsed: () => void }) => unknown) =>
    selector({ setSidebarCollapsed: () => undefined }),
}))
vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: false }),
}))
vi.mock('@/lib/auth/useCurrentUser', () => ({
  useCurrentUser: () => ({ firstName: '', lastName: '', email: '' }),
}))
vi.mock('@/lib/auth/useCurrentUserAvatar', () => ({ useCurrentUserAvatar: () => undefined }))
vi.mock('./Navigation', () => ({ default: () => null }))
vi.mock('./Breadcrumb', () => ({ default: () => null }))
vi.mock('../handbook/ContextHelpLink', () => ({ default: () => null }))

const { default: Header } = await import('./Header')

describe('Header', () => {
  // The map's bottom drawer is portaled to the end of body on z-50, so the
  // open mobile nav must sit above that layer or the drawer covers it.
  it('lifts the header above z-50 portals only while the mobile nav is open', async () => {
    const user = userEvent.setup()
    const { container } = render(<Header />)
    const header = container.querySelector('header')
    const toggle = screen.getByRole('button', { expanded: false })

    expect(header).toHaveClass('z-50')
    expect(header).not.toHaveClass('z-[60]')

    await user.click(toggle)
    expect(header).toHaveClass('z-[60]')
    expect(header).not.toHaveClass('z-50')

    await user.click(toggle)
    expect(header).toHaveClass('z-50')
  })
})
