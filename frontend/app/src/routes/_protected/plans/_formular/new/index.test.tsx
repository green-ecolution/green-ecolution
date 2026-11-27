import { render } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { probeManifest } from '@/test/planningManifest'
import useStore from '@/store/store'

const search = vi.hoisted(() => ({ value: {} }))
interface FormArgs {
  initForm: { module: string; description?: string }
  planManifest: { key: string }
}
const formSpy = vi.hoisted(() => vi.fn<(args: FormArgs) => void>())
const vehicleListSpy = vi.hoisted(() => vi.fn((_params?: unknown) => ({})))

vi.mock('@tanstack/react-router', () => ({
  createFileRoute: () => (options: { component: unknown }) => ({
    ...options,
    useSearch: () => search.value,
  }),
}))
vi.mock('@tanstack/react-query', () => ({
  useSuspenseQuery: () => ({ data: { data: [] } }),
}))
vi.mock('@/api/queries', () => ({
  userQueries: { list: () => ({}) },
  vehicleQueries: { list: vehicleListSpy },
}))
vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: [probeManifest('irrigation'), probeManifest('waste')] }),
}))
vi.mock('@/hooks/form/usePlanForm', () => ({
  usePlanForm: (
    _: string,
    args: { initForm: { module: string }; planManifest: { key: string } },
  ) => {
    formSpy(args)
    return {
      mutate: vi.fn(),
      isError: false,
      error: null,
      form: {},
      navigationBlocker: {},
      saveDraft: vi.fn(),
    }
  },
}))
vi.mock('react-hook-form', async (orig) => ({
  ...(await orig<typeof import('react-hook-form')>()),
  FormProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))
vi.mock('@/components/general/form/FormForPlan', () => ({ default: () => null }))
vi.mock('@/components/general/form/UnsavedChangesDialog', () => ({ default: () => null }))
vi.mock('@/components/general/links/BackLink', () => ({ default: () => null }))

import { Route } from './index'

const NewPlan = (Route as unknown as { component: () => React.JSX.Element }).component

describe('/plans/new module choice', () => {
  beforeEach(() => {
    formSpy.mockClear()
    useStore.getState().clearAllFormDrafts()
    useStore.getState().setFormDraft('plan-create', {
      module: 'waste',
      description: 'stored',
      stopIds: ['s1'],
    })
  })

  it('restores the stored draft of another module when no module is requested', () => {
    search.value = {}
    render(<NewPlan />)
    const args = formSpy.mock.lastCall![0]
    expect(args.planManifest.key).toBe('waste')
    expect(args.initForm.description).toBe('stored')
  })

  it('starts a fresh form when the requested module differs from the draft', () => {
    search.value = { module: 'irrigation' }
    render(<NewPlan />)
    const args = formSpy.mock.lastCall![0]
    expect(args.planManifest.key).toBe('irrigation')
    expect(args.initForm.module).toBe('irrigation')
    expect(args.initForm.description).toBe('')
  })

  it('loads transporters and trailers as separate, type-filtered lists', () => {
    search.value = { module: 'irrigation' }
    vehicleListSpy.mockClear()
    render(<NewPlan />)
    const requestedTypes = vehicleListSpy.mock.calls.map(
      ([params]) => (params as { type?: string[] } | undefined)?.type,
    )
    expect(requestedTypes).toEqual(expect.arrayContaining([['transporter'], ['trailer']]))
    expect(requestedTypes).toHaveLength(2)
  })
})
