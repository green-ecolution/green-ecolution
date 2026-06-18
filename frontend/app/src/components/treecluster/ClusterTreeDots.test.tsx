import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import ClusterTreeDots from './ClusterTreeDots'

describe('ClusterTreeDots', () => {
  afterEach(cleanup)

  it('caps at 24 dots, highlights sensor trees, and shows the overflow count', () => {
    const { container } = render(<ClusterTreeDots treeCount={30} sensorCount={2} />)

    const allDots = container.querySelectorAll('[data-dot]')
    const sensorDots = container.querySelectorAll('[data-dot="sensor"]')

    expect(allDots).toHaveLength(24)
    expect(sensorDots).toHaveLength(2)
    expect(screen.getByText('+6')).toBeInTheDocument()
  })

  it('renders the legend when sensors exist', () => {
    render(<ClusterTreeDots treeCount={10} sensorCount={3} />)
    expect(screen.getByText(/Sensor-Baum · misst die Gruppe/)).toBeInTheDocument()
  })

  it('omits the legend when there are no sensors', () => {
    render(<ClusterTreeDots treeCount={10} sensorCount={0} />)
    expect(screen.queryByText(/Sensor-Baum · misst die Gruppe/)).not.toBeInTheDocument()
  })
})
