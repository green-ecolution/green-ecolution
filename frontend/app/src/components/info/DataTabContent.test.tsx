import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { DataStatisticsResponse } from '@green-ecolution/backend-client'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import { render, screen } from '@/test/utils'
import type { ModuleManifest } from '@/modules/types'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: ReactNode }) => <a href={to}>{children}</a>,
}))

// recharts measures its container, which jsdom never lays out.
vi.mock('@green-ecolution/ui', async () => {
  const actual = await vi.importActual<typeof import('@green-ecolution/ui')>('@green-ecolution/ui')
  return { ...actual, ChartContainer: () => null }
})

const activeManifests = vi.fn((): ModuleManifest[] => [irrigationManifest])
vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests(), isActive: () => true }),
}))

const { default: DataTabContent } = await import('./DataTabContent')

const stats: DataStatisticsResponse = {
  sensorCount: 4,
  vehicleCount: 2,
  planCount: 3,
  modules: [
    { module: 'irrigation', kind: 'tree', count: 1200 },
    { module: 'irrigation', kind: 'cluster', count: 30 },
    { module: 'waste', kind: 'bin', count: 7 },
  ],
}

describe('DataTabContent', () => {
  beforeEach(() => activeManifests.mockReturnValue([irrigationManifest]))

  it('renders the irrigation counts from the manifest statistics', () => {
    render(<DataTabContent statsData={stats} />)

    expect(screen.getByText('Bäume gesamt')).toBeInTheDocument()
    expect(screen.getByText('1.200')).toBeInTheDocument()
    expect(screen.getByText('in 30 Bewässerungsgruppen verwaltet')).toBeInTheDocument()
    expect(screen.getByText('Verwaltete Bewässerungsgruppen')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Alle Bewässerungsgruppen ansehen/ })).toHaveAttribute(
      'href',
      '/treecluster',
    )
    expect(screen.getByText('1.239')).toBeInTheDocument()
  })

  it('shows no irrigation figures when the module is not active', () => {
    activeManifests.mockReturnValue([])
    render(<DataTabContent statsData={stats} />)

    expect(screen.queryByText('Bäume gesamt')).not.toBeInTheDocument()
    expect(screen.queryByText('Verwaltete Bewässerungsgruppen')).not.toBeInTheDocument()
    expect(screen.getByText('Aktive Messgeräte')).toBeInTheDocument()
  })

  it('renders a statistic of another module from its own manifest', () => {
    const waste: ModuleManifest = {
      key: 'waste',
      labelKey: 'common:modules.irrigation',
      i18n: { namespaces: [], load: () => Promise.resolve({}) },
      statistics: {
        bin: {
          icon: () => null,
          labelKey: 'common:modules.irrigation',
          subtitleKey: 'info:data.stat.sensorsSubtitle',
          to: '/bins',
        },
      },
    }
    activeManifests.mockReturnValue([waste])
    render(<DataTabContent statsData={stats} />)

    const link = screen.getAllByRole('link').find((a) => a.getAttribute('href') === '/bins')
    expect(link).toHaveTextContent('Bewässerung')
    expect(link).toHaveTextContent('Aktive Messgeräte')
  })
})
