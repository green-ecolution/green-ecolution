import { useState } from 'react'
import { beforeEach, describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'

const state = vi.hoisted(
  (): {
    orgId: string
    pathname: string
    permsByOrg: Partial<Record<string, Permissions>> | undefined
  } => ({ orgId: 'amt', pathname: '/settings/organization/amt/roles', permsByOrg: undefined }),
)

vi.mock('@tanstack/react-router', () => {
  const Counter = () => {
    const [count, setCount] = useState(0)
    return <button onClick={() => setCount(count + 1)}>clicks {count}</button>
  }
  return {
    getRouteApi: () => ({ useParams: () => ({ orgId: state.orgId }) }),
    useLocation: () => ({ pathname: state.pathname }),
    Outlet: Counter,
  }
})
vi.mock('@/lib/auth/usePermissionsIn', () => ({
  usePermissionsResolver: () =>
    state.permsByOrg &&
    ((orgId: string | null) =>
      (orgId === null ? undefined : state.permsByOrg?.[orgId]) ?? new Set<string>()),
}))
vi.mock('@/components/layout/Forbidden', () => ({ default: () => <p>forbidden</p> }))

const { default: SettingsOrganizationOutlet } = await import('./SettingsOrganizationOutlet')

describe('SettingsOrganizationOutlet', () => {
  beforeEach(() => {
    state.orgId = 'amt'
    state.pathname = '/settings/organization/amt/roles'
    state.permsByOrg = { amt: UNRESTRICTED, nord: UNRESTRICTED }
  })

  it('keeps the page state while the organization stays the same', async () => {
    const { rerender } = render(<SettingsOrganizationOutlet />)
    await userEvent.click(screen.getByRole('button'))

    rerender(<SettingsOrganizationOutlet />)

    expect(screen.getByRole('button')).toHaveTextContent('clicks 1')
  })

  it('remounts the page when the organization changes', async () => {
    const { rerender } = render(<SettingsOrganizationOutlet />)
    await userEvent.click(screen.getByRole('button'))

    state.orgId = 'nord'
    rerender(<SettingsOrganizationOutlet />)

    expect(screen.getByRole('button')).toHaveTextContent('clicks 0')
  })

  it('renders the page with the right in the selected organization', () => {
    state.permsByOrg = { amt: new Set(['role:read']) }
    render(<SettingsOrganizationOutlet />)
    expect(screen.getByRole('button')).toBeInTheDocument()
  })

  it('is forbidden without the right in the selected organization', () => {
    state.permsByOrg = { amt: new Set(['organization:read']), nord: new Set(['role:read']) }
    render(<SettingsOrganizationOutlet />)
    expect(screen.getByText('forbidden')).toBeInTheDocument()
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('requires organization:read for the overview', () => {
    state.pathname = '/settings/organization/amt'
    state.permsByOrg = { amt: new Set(['start_point:read']) }
    render(<SettingsOrganizationOutlet />)
    expect(screen.getByText('forbidden')).toBeInTheDocument()
  })

  it('checks a detail page against its list', () => {
    state.pathname = '/settings/organization/amt/plugins/kataster'
    state.permsByOrg = { amt: new Set(['role:read']) }
    render(<SettingsOrganizationOutlet />)
    expect(screen.getByText('forbidden')).toBeInTheDocument()
  })

  it('renders nothing while the rights load', () => {
    state.permsByOrg = undefined
    render(<SettingsOrganizationOutlet />)
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
    expect(screen.queryByText('forbidden')).not.toBeInTheDocument()
  })
})
