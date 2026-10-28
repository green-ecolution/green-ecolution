import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen } from '@testing-library/react'
import type { SensorResponse } from '@green-ecolution/backend-client'
import { wasteBinApi } from '../../api/client'
import { renderInRouter } from '../../test/renderInRouter'
import { BIN_FULL_ID, wasteBinFixtures } from '../../test/handlers'
import BinPlacement from './BinPlacement'

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children }: { children?: React.ReactNode }) => (
    <div data-testid="map">{children}</div>
  ),
}))
vi.mock('@/components/map-gl/SensorMarker', () => ({ default: () => null }))

const sensor = {
  id: 'eui-1',
  target: { module: 'waste', kind: 'bin', id: BIN_FULL_ID },
  location: { latitude: 54.78, longitude: 9.43 },
} as unknown as SensorResponse

describe('BinPlacement', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.spyOn(wasteBinApi, 'getWasteBin').mockResolvedValue(wasteBinFixtures[0])
  })

  it('links to the bin the sensor sits at', async () => {
    renderInRouter(<BinPlacement sensor={sensor} />)

    const link = await screen.findByRole('link')
    expect(link).toHaveAttribute('href', `/waste-bins/${BIN_FULL_ID}`)
    expect(link).toHaveTextContent('B-0042')
    expect(link).toHaveTextContent('Voll · 85 %')
  })

  it('draws the map and the core’s coordinates panel', async () => {
    renderInRouter(
      <BinPlacement sensor={sensor}>
        <p>Koordinaten</p>
      </BinPlacement>,
    )

    expect(await screen.findByTestId('map')).toBeInTheDocument()
    expect(screen.getByText('Koordinaten')).toBeInTheDocument()
  })

  it('says so when the bin cannot be loaded', async () => {
    vi.spyOn(wasteBinApi, 'getWasteBin').mockRejectedValue(new Error('boom'))
    renderInRouter(<BinPlacement sensor={sensor} />)

    expect(await screen.findByText('Behälter nicht geladen')).toBeInTheDocument()
  })
})
