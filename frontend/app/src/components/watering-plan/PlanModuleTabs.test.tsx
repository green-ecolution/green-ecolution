import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import { render, screen } from '@/test/utils'
import { probeManifest } from '@/test/planningManifest'
import type { ModuleManifest } from '@/modules/types'

vi.mock('@tanstack/react-router', () => ({
  Link: ({
    children,
    to,
    search,
    ...rest
  }: {
    children: ReactNode
    to: string
    search: { module: string }
  }) => (
    <a href={`${to}?module=${search.module}`} {...rest}>
      {children}
    </a>
  ),
}))

const activeManifests = vi.fn((): ModuleManifest[] => [])
vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests(), isActive: () => true }),
}))

const { default: PlanModuleTabs } = await import('./PlanModuleTabs')

const irrigation = probeManifest('irrigation', { labelKey: 'wateringPlan:list.title' })
const waste = probeManifest('waste', { labelKey: 'wateringPlan:list.createButton' })

describe('PlanModuleTabs', () => {
  beforeEach(() => activeManifests.mockReset())

  it('hides itself while only one module plans', () => {
    activeManifests.mockReturnValue([irrigation])
    render(<PlanModuleTabs current="irrigation" />)

    expect(screen.queryByRole('navigation')).not.toBeInTheDocument()
  })

  it('links every planning module and marks the current one', () => {
    activeManifests.mockReturnValue([irrigation, waste])
    render(<PlanModuleTabs current="waste" />)

    expect(screen.getByRole('navigation', { name: 'Einsatzart' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Neuen Einsatzplan erstellen' })).toHaveAttribute(
      'aria-current',
      'page',
    )
    const other = screen.getByRole('link', { name: 'Einsatzpläne' })
    expect(other).toHaveAttribute('href', '/plans?module=irrigation')
    expect(other).not.toHaveAttribute('aria-current')
  })
})
