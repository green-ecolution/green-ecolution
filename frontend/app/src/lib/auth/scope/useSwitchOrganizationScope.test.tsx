import { describe, it, expect, vi, beforeEach } from 'vitest'
import { memo } from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query'
import type { UserResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { getOrganizationScope, setOrganizationScope } from './organizationScopeStore'
import { useSwitchOrganizationScope } from './useSwitchOrganizationScope'

const invalidate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({ useRouter: () => ({ invalidate }) }))

// Memoized and without a scope subscription: it re-renders only when its
// query observer reports new data.
const ScopedData = memo(function ScopedData() {
  const { data } = useQuery({
    queryKey: ['scoped'],
    queryFn: () => Promise.resolve(`data:${getOrganizationScope() ?? 'all'}`),
    staleTime: Infinity,
  })
  return <p>{data ?? 'loading'}</p>
})

function SwitchButton() {
  const switchScope = useSwitchOrganizationScope()
  return <button onClick={() => void switchScope('firma_a')}>switch</button>
}

const renderSwitch = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false } },
  })
  queryClient.setQueryData(userQueries.me().queryKey, { id: 'u1' } as UserResponse)
  queryClient.setQueryData(userQueries.myOrganizations().queryKey, [])
  render(
    <QueryClientProvider client={queryClient}>
      <ScopedData />
      <SwitchButton />
    </QueryClientProvider>,
  )
  return queryClient
}

describe('useSwitchOrganizationScope', () => {
  beforeEach(() => {
    localStorage.clear()
    setOrganizationScope('u1', null)
    vi.clearAllMocks()
  })

  it('refetches the data a mounted component shows', async () => {
    renderSwitch()
    expect(await screen.findByText('data:all')).toBeInTheDocument()

    await userEvent.setup().click(screen.getByText('switch'))

    expect(await screen.findByText('data:firma_a')).toBeInTheDocument()
    await waitFor(() => expect(invalidate).toHaveBeenCalled())
  })

  it('keeps the queries the backend answers without the scope', async () => {
    const queryClient = renderSwitch()
    await screen.findByText('data:all')

    await userEvent.setup().click(screen.getByText('switch'))

    await waitFor(() => expect(invalidate).toHaveBeenCalled())
    expect(queryClient.getQueryData(userQueries.me().queryKey)).toEqual({ id: 'u1' })
    expect(queryClient.getQueryData(userQueries.myOrganizations().queryKey)).toEqual([])
  })
})
