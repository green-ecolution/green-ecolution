import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { DataHealth } from '@green-ecolution/backend-client'
import TreeSensorCard from './TreeSensorCard'
import type { Tree } from '@/api/backendApi'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => <a href="/">{children}</a>,
}))

const treeWith = (dataHealth: DataHealth, implausibleRecent: number) =>
  ({
    sensor: {
      id: 'eui-test',
      status: 'online',
      dataHealth,
      implausibleRecent,
    },
  }) as unknown as Tree

const WARNING = /zweifelhaften Sensordaten/

describe('TreeSensorCard', () => {
  it('warns when the sensor data is suspect', () => {
    render(<TreeSensorCard tree={treeWith(DataHealth.Suspect, 4)} />)
    expect(screen.getByText(WARNING)).toBeInTheDocument()
  })

  it('warns when values were flagged without a defect suspicion', () => {
    render(<TreeSensorCard tree={treeWith(DataHealth.Ok, 2)} />)
    expect(screen.getByText(WARNING)).toBeInTheDocument()
  })

  it('stays quiet on clean data', () => {
    render(<TreeSensorCard tree={treeWith(DataHealth.Ok, 0)} />)
    expect(screen.queryByText(WARNING)).not.toBeInTheDocument()
  })

  it('says nothing about data quality when the tree has no sensor', () => {
    render(<TreeSensorCard tree={{} as Tree} />)
    expect(screen.getByText(/keinem Sensor ausgestattet/)).toBeInTheDocument()
    expect(screen.queryByText(WARNING)).not.toBeInTheDocument()
  })
})
