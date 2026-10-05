import { describe, expect, it, vi } from 'vitest'
import { render, screen, within } from '@testing-library/react'
import SensorMeasurementProfile from './SensorMeasurementProfile'
import type { Sensor } from '@/api/backendApi'

const { useQueryMock } = vi.hoisted(() => ({ useQueryMock: vi.fn() }))

vi.mock('@tanstack/react-query', () => ({
  useQuery: useQueryMock,
  queryOptions: (options: unknown) => options,
}))

const sensor = { id: 'eui-test', model: { id: 'model-1', name: 'SMT100' } } as Sensor

describe('SensorMeasurementProfile', () => {
  it('lists each depth once, shallowest first, with everything measured there', () => {
    useQueryMock.mockReturnValue({
      data: {
        abilities: [
          { ability: 'soil_moisture', unit: 'percent', depthCm: 80 },
          { ability: 'soil_moisture', unit: 'percent', depthCm: 40 },
          { ability: 'temperature', unit: 'celsius', depthCm: 40 },
        ],
      },
      isLoading: false,
      isError: false,
    })

    render(<SensorMeasurementProfile sensor={sensor} />)

    const [surface, shallow, deep] = screen
      .getAllByRole('listitem')
      .filter((li) => li.parentElement?.tagName === 'OL')
    expect(surface).toHaveTextContent('Bodenoberfläche')
    expect(shallow).toHaveTextContent('40 cm')
    expect(within(shallow).getByText('Bodenfeuchtigkeit')).toBeInTheDocument()
    expect(within(shallow).getByText('Temperatur')).toBeInTheDocument()
    expect(deep).toHaveTextContent('80 cm')
    expect(within(deep).queryByText('Temperatur')).not.toBeInTheDocument()
  })

  it('renders nothing for a model without abilities', () => {
    useQueryMock.mockReturnValue({ data: { abilities: [] }, isLoading: false, isError: false })

    const { container } = render(<SensorMeasurementProfile sensor={sensor} />)

    expect(container).toBeEmptyDOMElement()
  })
})
