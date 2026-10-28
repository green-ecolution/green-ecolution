import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@green-ecolution/ui'
import type { ReachableOrganizationResponse, UserResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { getOrganizationScope, setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import OrganizationScopeMenu from './OrganizationScopeMenu'

const invalidate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({ useRouter: () => ({ invalidate }) }))

const ROOT = { id: 'root', name: 'Stadt', parentId: null }
const A = { id: 'firma_a', name: 'Firma A', parentId: 'root' }
const B = { id: 'firma_b', name: 'Firma B', parentId: 'root' }
const A1 = { id: 'a1', name: 'Abteilung Grün', parentId: 'firma_a' }

const renderMenu = (orgs: ReachableOrganizationResponse[]) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity } } })
  queryClient.setQueryData(userQueries.myOrganizations().queryKey, orgs)
  queryClient.setQueryData(userQueries.me().queryKey, { id: 'u1' } as UserResponse)
  const clear = vi.spyOn(queryClient, 'clear')
  render(
    <QueryClientProvider client={queryClient}>
      <DropdownMenu>
        <DropdownMenuTrigger>open</DropdownMenuTrigger>
        <DropdownMenuContent>
          <OrganizationScopeMenu />
        </DropdownMenuContent>
      </DropdownMenu>
    </QueryClientProvider>,
  )
  return { clear }
}

const openSub = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.click(screen.getByText('open'))
  const trigger = await screen.findByRole('menuitem', { name: /organisation/i })
  trigger.focus()
  await user.keyboard('{ArrowRight}')
}

describe('OrganizationScopeMenu', () => {
  beforeEach(() => {
    localStorage.clear()
    setOrganizationScope('u1', null)
    vi.clearAllMocks()
  })

  it('renders nothing with a single reachable organization', async () => {
    renderMenu([ROOT])
    await userEvent.setup().click(screen.getByText('open'))
    await waitFor(() => expect(screen.getByRole('menu')).toBeInTheDocument())
    expect(screen.queryByRole('menuitem', { name: /organisation/i })).not.toBeInTheDocument()
  })

  it('lists all organizations plus the tree with the active entry checked', async () => {
    setOrganizationScope('u1', 'firma_a')
    renderMenu([ROOT, A, B, A1])
    await openSub(userEvent.setup())

    const items = await screen.findAllByRole('menuitemradio')
    expect(items.map((i) => i.textContent)).toEqual([
      'Alle meine Organisationen',
      'Stadt',
      'Firma A',
      'Abteilung Grün',
      'Firma B',
    ])
    expect(screen.getByRole('menuitemradio', { name: 'Firma A' })).toHaveAttribute(
      'aria-checked',
      'true',
    )
    expect(screen.getByRole('menuitemradio', { name: 'Firma B' })).toHaveAttribute(
      'aria-checked',
      'false',
    )
  })

  it('stores the scope, clears the cache and invalidates the router on selection', async () => {
    const user = userEvent.setup()
    const { clear } = renderMenu([ROOT, A, B])
    await openSub(user)
    await user.click(await screen.findByRole('menuitemradio', { name: 'Firma A' }))

    await waitFor(() => expect(invalidate).toHaveBeenCalled())
    expect(getOrganizationScope()).toBe('firma_a')
    expect(localStorage.getItem('ge.organizationScope.u1')).toBe('firma_a')
    expect(clear).toHaveBeenCalled()
  })

  it('shows the search field only above eight entries and filters with ancestors', async () => {
    const user = userEvent.setup()
    const many = Array.from({ length: 7 }, (_, i) => ({
      id: `x${i}`,
      name: `Amt ${i}`,
      parentId: 'root',
    }))
    renderMenu([ROOT, A, B, A1, ...many])
    await openSub(user)

    const search = await screen.findByRole('searchbox', { name: /organisation suchen/i })
    await user.type(search, 'abteilung')

    const items = screen.getAllByRole('menuitemradio')
    expect(items.map((i) => i.textContent)).toEqual([
      'Alle meine Organisationen',
      'Stadt',
      'Firma A',
      'Abteilung Grün',
    ])
  })

  it('has no search field for a short list', async () => {
    renderMenu([ROOT, A, B])
    await openSub(userEvent.setup())
    await screen.findAllByRole('menuitemradio')
    expect(screen.queryByRole('textbox')).not.toBeInTheDocument()
  })
})
