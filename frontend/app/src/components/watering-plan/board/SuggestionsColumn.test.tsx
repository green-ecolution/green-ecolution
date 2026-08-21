/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const cluster = {
  id: 'c1',
  name: 'Am Hafen',
  treeIds: ['t1', 't2'],
  wateringStatus: 'bad',
}

vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>()
  return {
    ...actual,
    useQuery: () => ({ data: { data: [cluster] }, isError: false, refetch: vi.fn() }),
  }
})

const { default: SuggestionsColumn } = await import('./SuggestionsColumn')

describe('SuggestionsColumn selection gating', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  it('offers no selection without watering_plan:create', async () => {
    permissions.mockReturnValue(new Set(['watering_plan:read', 'tree_cluster:read']))
    render(<SuggestionsColumn />)

    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /bündeln/i })).not.toBeInTheDocument()

    await userEvent.click(screen.getByText('Am Hafen').closest('div')!)
    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument()
  })

  it('keeps the suggestion readable without watering_plan:create', () => {
    permissions.mockReturnValue(new Set(['watering_plan:read', 'tree_cluster:read']))
    render(<SuggestionsColumn />)

    expect(screen.getByText('Am Hafen')).toBeInTheDocument()
    expect(screen.getByText('2 Bäume')).toBeInTheDocument()
  })

  it('allows selecting and bundling with watering_plan:create', async () => {
    permissions.mockReturnValue(new Set(['watering_plan:create']))
    render(<SuggestionsColumn />)

    const bundle = screen.getByRole('button', { name: /bündeln/i })
    expect(bundle).toBeDisabled()

    await userEvent.click(screen.getByRole('checkbox'))
    expect(screen.getByRole('checkbox')).toBeChecked()
    expect(screen.getByRole('button', { name: /bündeln/i })).toBeEnabled()
  })
})
