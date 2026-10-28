import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { GeolocationFix } from '@/hooks/useGeolocation'
import { wasteBinApi } from '../../../api/client'
import { wasteBinFixtures } from '../../../test/handlers'
import SensorBinStep from './SensorBinStep'

vi.mock('@/components/geolocation/LocationMapPreview', () => ({
  default: () => <div data-testid="location-map-preview" />,
}))

// Closest to B-0043 (54.7801, 9.4402), the fixture B-0042 lies further north-west.
const fix: GeolocationFix = {
  latitude: 54.78,
  longitude: 9.44,
  accuracy: 7.5,
  altitude: null,
  altitudeAccuracy: null,
  heading: null,
  speed: null,
  timestamp: 1_710_000_000_000,
}

const renderStep = (props: Partial<React.ComponentProps<typeof SensorBinStep>> = {}) =>
  render(
    <SensorBinStep
      position={fix}
      geolocationStatus="watching"
      geolocationError={null}
      current={undefined}
      onSelect={vi.fn()}
      onRelocate={vi.fn()}
      {...props}
    />,
  )

describe('SensorBinStep', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    vi.spyOn(wasteBinApi, 'listWasteBins').mockResolvedValue({
      data: wasteBinFixtures,
      pagination: { currentPage: 1, perPage: 100, totalPages: 1, totalRecords: 2 },
    })
  })

  it('lists the bins nearest to the position first', async () => {
    renderStep()

    const rows = await screen.findAllByRole('button', { name: /B-004/ })
    expect(rows.map((row) => /B-\d{4}/.exec(row.textContent ?? '')?.[0])).toEqual([
      'B-0043',
      'B-0042',
    ])
  })

  it('hands the picked bin to the core as a selected target', async () => {
    const onSelect = vi.fn()
    renderStep({ onSelect })

    await userEvent.click(await screen.findByRole('button', { name: /B-0043/ }))

    expect(onSelect).toHaveBeenCalledWith({
      target: { module: 'waste', kind: 'bin', id: wasteBinFixtures[1].id },
      label: 'B-0043',
    })
  })

  it('keeps the search reachable without a position', async () => {
    renderStep({ position: null, geolocationStatus: 'denied' })

    expect(screen.getByText(/Standortzugriff verweigert/i)).toBeInTheDocument()
    await userEvent.type(await screen.findByRole('searchbox'), '0042')
    expect(screen.queryByRole('button', { name: /B-0043/ })).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: /B-0042/ })).toBeInTheDocument()
  })
})
