import type { ReactNode } from 'react'
import type { ErrorComponentProps } from '@tanstack/react-router'
import type { QueryClient } from '@tanstack/react-query'
import { modulesQuery } from '@/api/queries'
import ErrorFallback from '@/components/layout/ErrorFallback'
import { activeManifestsFrom } from './activeModules'
import ModuleInactive from './ModuleInactive'
import { moduleManifests } from './registry'

export class ModuleInactiveError extends Error {
  constructor(readonly moduleKey: string) {
    super(`Module ${moduleKey} is not active`)
    this.name = 'ModuleInactiveError'
  }
}

interface BeforeLoadContext {
  context: { queryClient: QueryClient }
}

interface ModuleRouteOptions {
  beforeLoad?: (ctx: BeforeLoadContext) => unknown
  errorComponent?: (props: ErrorComponentProps) => ReactNode
}

/**
 * Wraps a module's top-level route: an inactive module throws before the
 * wrapped guard and the child loaders run, so no module API request is made.
 */
export const moduleRoute = <T extends object & ModuleRouteOptions>(
  moduleKey: string,
  options: T,
): Omit<T, 'beforeLoad' | 'errorComponent'> & {
  beforeLoad: (ctx: BeforeLoadContext) => Promise<unknown>
  errorComponent: (props: ErrorComponentProps) => ReactNode
} => ({
  ...options,
  beforeLoad: async (ctx) => {
    const modules = await ctx.context.queryClient.ensureQueryData(modulesQuery())
    if (!activeManifestsFrom(moduleManifests, modules).some((m) => m.key === moduleKey)) {
      throw new ModuleInactiveError(moduleKey)
    }
    return options.beforeLoad?.(ctx)
  },
  errorComponent: (props) => {
    if (props.error instanceof ModuleInactiveError) return <ModuleInactive />
    if (options.errorComponent) return options.errorComponent(props)
    return <ErrorFallback error={props.error} resetErrorBoundary={props.reset} />
  },
})
