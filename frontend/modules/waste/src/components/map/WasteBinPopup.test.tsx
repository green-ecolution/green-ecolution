import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { server } from '@/test/mocks/server'
import { renderInRouter } from '../../test/renderInRouter'
import { BIN_FULL_ID, wasteBinFixtures, wasteHandlers } from '../../test/handlers'
import WasteBinPopup, { WasteBinPopupView } from './WasteBinPopup'

// The real session needs OIDC settings the test environment does not have.
vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

describe('WasteBinPopupView', () => {
  it('shows number, fill state and a link to the detail page', async () => {
    renderInRouter(<WasteBinPopupView bin={wasteBinFixtures[0]} onClose={() => undefined} />)

    expect(await screen.findByRole('heading', { name: 'Behälter B-0042' })).toBeInTheDocument()
    expect(screen.getByText('Voll · 85 %')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Zum Behälter' })).toHaveAttribute(
      'href',
      `/waste-bins/${BIN_FULL_ID}`,
    )
  })

  it('reports a close click', async () => {
    const onClose = vi.fn()
    renderInRouter(<WasteBinPopupView bin={wasteBinFixtures[0]} onClose={onClose} />)

    await userEvent.click(await screen.findByRole('button', { name: 'Schließen' }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})

describe('WasteBinPopup', () => {
  beforeEach(() => {
    server.use(...wasteHandlers)
  })

  it('loads and shows the bin', async () => {
    renderInRouter(<WasteBinPopup binId={BIN_FULL_ID} onClose={() => undefined} />)

    expect(await screen.findByRole('heading', { name: 'Behälter B-0042' })).toBeInTheDocument()
  })

  it('says the bin was not found for an id that is no uuid', async () => {
    const onClose = vi.fn()
    renderInRouter(<WasteBinPopup binId="not-a-uuid" onClose={onClose} />)

    expect(await screen.findByText('Behälter nicht gefunden')).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Schließen' }))
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('says the bin was not found when the API answers 404', async () => {
    renderInRouter(
      <WasteBinPopup binId="0190a8e9-7c4f-7000-8000-00000000beef" onClose={() => undefined} />,
    )

    expect(await screen.findByText('Behälter nicht gefunden')).toBeInTheDocument()
  })
})
