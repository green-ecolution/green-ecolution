import type { ComponentProps, ReactNode } from 'react'
import { Outlet } from '@tanstack/react-router'
import type { ErrorComponentProps } from '@tanstack/react-router'
import type { FetchQueryOptions, QueryClient, QueryKey } from '@tanstack/react-query'
import { Loading } from '@green-ecolution/ui'
import EntityNotFound from '@/components/layout/EntityNotFound'
import ErrorFallback from '@/components/layout/ErrorFallback'
import Forbidden from '@/components/layout/Forbidden'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import {
  permissionsOf,
  satisfies,
  UNRESTRICTED,
  type PermissionRequirement,
} from '@/lib/auth/permissions'

/** Options for layout routes that only render an Outlet and contribute a breadcrumb. */
export const crumbRoute = (title: string) => ({
  component: Outlet,
  loader: () => ({ crumb: { title } }),
})

export const pendingLoading = (label: string) => () => (
  <Loading className="mt-20 justify-center" label={label} />
)

export const entityNotFound = (props: ComponentProps<typeof EntityNotFound>) => () => (
  <EntityNotFound {...props} />
)

/** Fire-and-forget prefetch for route loaders; failures surface via the query itself. */
export const prefetch = <TQueryFnData, TError, TData, TQueryKey extends QueryKey>(
  queryClient: QueryClient,
  options: FetchQueryOptions<TQueryFnData, TError, TData, TQueryKey>,
  label: string,
): void => {
  queryClient
    .prefetchQuery(options)
    .catch((error: unknown) => console.error(`Prefetching "${label}" failed:`, error))
}

interface EntityRouteOptions<TEntity, TKey extends string> {
  key: TKey
  query: (id: string) => FetchQueryOptions<TEntity>
  /** Name of the path param carrying the entity id, e.g. 'treeId'. */
  idParam: string
  title: (entity: TEntity) => string
  notFound: ComponentProps<typeof EntityNotFound>
}

/**
 * Options for detail layout routes: fetch the entity, expose it as loader data
 * under `key` together with a breadcrumb, and render EntityNotFound on failure.
 */
export const entityRoute = <TEntity, TKey extends string>({
  key,
  query,
  idParam,
  title,
  notFound,
}: EntityRouteOptions<TEntity, TKey>) => ({
  component: Outlet,
  loader: async ({
    context: { queryClient },
    params,
  }: {
    context: { queryClient: QueryClient }
    params: Record<string, string>
  }) => {
    const entity = await queryClient.fetchQuery(query(params[idParam]))
    return {
      [key]: entity,
      crumb: { title: title(entity) },
    } as Record<TKey, TEntity> & { crumb: { title: string } }
  },
  errorComponent: () => <EntityNotFound {...notFound} />,
})

export class ForbiddenError extends Error {
  constructor() {
    super('Forbidden')
    this.name = 'ForbiddenError'
  }
}

/**
 * beforeLoad guard. Throwing here keeps the child route's loader from running,
 * so a denied route issues no API requests and shows no content first.
 */
export const requirePermission =
  (required: PermissionRequirement) =>
  async ({ context }: { context: { queryClient: QueryClient } }): Promise<void> => {
    const perms = readAuthBypass()
      ? UNRESTRICTED
      : permissionsOf(await context.queryClient.ensureQueryData(userQueries.me()))

    if (!satisfies(perms, required)) throw new ForbiddenError()
  }

export const forbiddenErrorComponent =
  (fallback?: (props: ErrorComponentProps) => ReactNode) =>
  (props: ErrorComponentProps): ReactNode => {
    if (props.error instanceof ForbiddenError) return <Forbidden />
    if (fallback) return fallback(props)
    return <ErrorFallback error={props.error} resetErrorBoundary={props.reset} />
  }

/**
 * Sugar for the crumbRoute-shaped cases; richer routes use the two primitives
 * directly. `beforeLoad?: undefined` rejects options carrying their own
 * beforeLoad, which the guard would otherwise drop silently.
 */
export const guardedRoute = <T extends object>(
  required: PermissionRequirement,
  options: T & {
    beforeLoad?: undefined
    errorComponent?: (props: ErrorComponentProps) => ReactNode
  },
) => ({
  ...options,
  beforeLoad: requirePermission(required),
  errorComponent: forbiddenErrorComponent(options.errorComponent),
})
