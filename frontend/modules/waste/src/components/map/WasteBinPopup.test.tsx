import { describe, expect, it, vi } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderInRouter } from '../../test/renderInRouter'
import { BIN_FULL_ID, wasteBinFixtures } from '../../test/handlers'
import { WasteBinPopupView } from './WasteBinPopup'

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
