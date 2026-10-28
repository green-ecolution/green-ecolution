import { describe, expect, it } from 'vitest'
import { screen } from '@testing-library/react'
import { renderInRouter } from '../test/renderInRouter'
import { BIN_FULL_ID, wasteBinFixtures } from '../test/handlers'
import WasteBinCard from './WasteBinCard'

describe('WasteBinCard', () => {
  it('links to the bin and shows number, volume and fill state', async () => {
    renderInRouter(<WasteBinCard bin={wasteBinFixtures[0]} />)

    const link = await screen.findByRole('link')
    expect(link).toHaveAttribute('href', `/waste-bins/${BIN_FULL_ID}`)
    expect(link).toHaveTextContent('B-0042')
    expect(link).toHaveTextContent('1100 l')
    expect(screen.getByText('Voll · 85 %')).toBeInTheDocument()
  })

  it('names the sensor when one is attached', async () => {
    renderInRouter(<WasteBinCard bin={wasteBinFixtures[0]} />)
    expect(await screen.findByText('eui-a84041000181c2f1')).toBeInTheDocument()
  })

  it('says so when no sensor is attached', async () => {
    renderInRouter(<WasteBinCard bin={wasteBinFixtures[1]} />)
    expect(await screen.findByText('Kein Sensor')).toBeInTheDocument()
  })
})
