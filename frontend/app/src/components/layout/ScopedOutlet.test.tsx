/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, beforeEach } from 'vitest'
import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
} from '@tanstack/react-router'
import { setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import ScopedOutlet from './ScopedOutlet'

const renderScoped = () => {
  const rootRoute = createRootRoute({ component: ScopedOutlet })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <input aria-label="Entwurf" />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return render(<RouterProvider router={router} />)
}

describe('ScopedOutlet', () => {
  beforeEach(() => {
    localStorage.clear()
    setOrganizationScope('u1', null)
  })

  it('remounts the page when the scope changes', async () => {
    const user = userEvent.setup()
    renderScoped()
    await user.type(await screen.findByLabelText('Entwurf'), 'Firma B')

    act(() => setOrganizationScope('u1', 'firma_a'))

    expect(await screen.findByLabelText('Entwurf')).toHaveValue('')
  })
})
