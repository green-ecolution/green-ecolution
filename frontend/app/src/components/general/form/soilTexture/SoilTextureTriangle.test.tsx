import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SoilCondition } from '@green-ecolution/backend-client'
import SoilTextureTriangle from './SoilTextureTriangle'

describe('SoilTextureTriangle', () => {
  it('renders an accessible diagram with all regions, labels and the marker', () => {
    render(<SoilTextureTriangle silt={25} clay={10} activeCondition={SoilCondition.Sl3} />)

    const diagram = screen.getByRole('img')
    expect(diagram).toHaveAccessibleName('Bodenartendiagramm: Schluff 25 %, Ton 10 %, Klasse Sl3')
    expect(screen.getByText('Sl3')).toBeInTheDocument()
    expect(screen.getByText('Tt')).toBeInTheDocument()
    expect(screen.getByTestId('soil-marker')).toBeInTheDocument()
    expect(screen.getByTestId('region-Sl3')).toHaveAttribute('data-active', 'true')
    expect(screen.getByTestId('region-Tt')).toHaveAttribute('data-active', 'false')
  })

  it('positions the marker at the silt/clay coordinates', () => {
    render(<SoilTextureTriangle silt={0} clay={0} activeCondition={SoilCondition.Ss} />)
    const marker = screen.getByTestId('soil-marker')
    // Plot origin: x = 14 + silt, y = 6 + (100 - clay)
    expect(marker).toHaveAttribute('cx', '14')
    expect(marker).toHaveAttribute('cy', '106')
  })
})
