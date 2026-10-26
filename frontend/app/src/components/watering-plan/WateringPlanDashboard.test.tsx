import { describe, it, expect, vi, beforeEach } from 'vitest'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { render, screen } from '@/test/utils'
import { probeManifest } from '@/test/planningManifest'
import { planResponse } from '@/test/plans'
import type { ModuleManifest } from '@/modules/types'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))
vi.mock('@/hooks/useDownloadGpx', () => ({ useDownloadGpx: () => ({ mutate: vi.fn() }) }))
vi.mock('@/lib/auth/useHasPermission', () => ({ useHasPermission: () => false }))
vi.mock('@/components/comments/CommentsSection', () => ({ default: () => null }))
vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>()
  return {
    ...actual,
    useSuspenseQuery: () => ({ data: { data: [] } }),
    useQuery: () => ({ data: [] }),
  }
})

const activeManifests = vi.fn((): ModuleManifest[] => [])
vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests(), isActive: () => true }),
}))

const { default: WateringPlanDashboard } = await import('./WateringPlanDashboard')

describe('WateringPlanDashboard', () => {
  beforeEach(() => {
    activeManifests.mockReturnValue([probeManifest('waste'), probeManifest('irrigation')])
  })

  it('renders the summary and the stop list of the plan module', async () => {
    const user = userEvent.setup()
    render(<WateringPlanDashboard wateringPlan={planResponse({ module: 'irrigation' })} />)

    expect(await screen.findByTestId('plan-summary-irrigation')).toBeInTheDocument()
    expect(screen.queryByTestId('plan-summary-waste')).not.toBeInTheDocument()

    await user.click(screen.getByRole('tab', { name: 'Bewässerungsgruppen' }))
    expect(await screen.findByTestId('stop-list-irrigation')).toBeInTheDocument()
    expect(screen.queryByTestId('stop-list-waste')).not.toBeInTheDocument()
  })

  it('leaves the module parts out when the plan module is not active', () => {
    activeManifests.mockReturnValue([probeManifest('waste')])
    render(<WateringPlanDashboard wateringPlan={planResponse({ module: 'irrigation' })} />)

    expect(screen.queryByTestId('plan-summary-waste')).not.toBeInTheDocument()
    expect(screen.queryByTestId('plan-summary-irrigation')).not.toBeInTheDocument()
    expect(screen.queryByRole('tab', { name: 'Bewässerungsgruppen' })).not.toBeInTheDocument()
  })
})
