import type { ComponentProps, ReactNode } from 'react'
import { Outlet } from '@tanstack/react-router'
import type { ErrorComponentProps } from '@tanstack/react-router'
import type { FetchQueryOptions, QueryClient, QueryKey } from '@tanstack/react-query'
import type { ParseKeys } from 'i18next'
import { Loading } from '@green-ecolution/ui'
import EntityNotFound from '@/components/layout/EntityNotFound'
import ErrorFallback from '@/components/layout/ErrorFallback'
import Forbidden from '@/components/layout/Forbidden'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import type { NavigationCrumbKey } from '@/lib/i18n/navigation'
import type { NAMESPACES } from '@/lib/i18n/languages'
import { useLocalizedText, type LocalizedText } from '@/lib/i18n/localizedText'
import {
  permissionsOf,
  satisfies,
  UNRESTRICTED,
  type PermissionRequirement,
} from '@/lib/auth/permissions'

/** Options for layout routes that only render an Outlet and contribute a breadcrumb. */
export const crumbRoute = (titleKey: NavigationCrumbKey) => ({
  component: Outlet,
  loader: () => ({ crumb: { titleKey } }),
})

/**
 * Route modules are imported eagerly (see `routeTree.gen.ts`), well before
 * `createI18n()` resolves, so `label` is a key resolved by `useLocalizedText()`
 * once this component actually renders — never pre-resolved at declaration
 * time, so it stays correct if the language changes while mounted. No current
 * caller passes a plain string — `label` takes the full `LocalizedText` union
 * (not a key-only type) so it stays usable for a future label built from
 * loaded data rather than a catalog key, the same reason `EntityCrumbTitle`
 * below keeps its own string branch for `treecluster.name`.
 */
export const pendingLoading = (label: LocalizedText) => {
  const PendingLoading = () => {
    const resolve = useLocalizedText()
    return <Loading className="mt-20 justify-center" label={resolve(label)} />
  }
  return PendingLoading
}

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

interface EntityCrumbKey {
  titleKey: ParseKeys<typeof NAMESPACES>
  params: Record<string, unknown>
}

/**
 * A translated title resolves reactively at render time (see `useBreadcrumbs`),
 * so most entities pass their key and interpolation params rather than a
 * pre-resolved string. Only a title built from data no catalog holds (a plain
 * entity name, e.g. `treecluster.name`) passes the literal string instead.
 */
export type EntityCrumbTitle = string | EntityCrumbKey

/**
 * Structurally identical to `@tanstack/react-router`'s ambient `Breadcrumb`
 * (declared in `main.tsx`) — duplicated rather than imported because a plain
 * `type` added to an external module via `declare module` augmentation isn't
 * re-exported as an importable named member, only merged into the ambient
 * scope it's declared in.
 */
type EntityCrumb = { title: string } | EntityCrumbKey

interface EntityRouteOptions<TEntity, TKey extends string> {
  key: TKey
  query: (id: string) => FetchQueryOptions<TEntity>
  /** Name of the path param carrying the entity id, e.g. 'treeId'. */
  idParam: string
  title: (entity: TEntity) => EntityCrumbTitle
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
    const resolvedTitle = title(entity)
    const crumb: EntityCrumb =
      typeof resolvedTitle === 'string' ? { title: resolvedTitle } : resolvedTitle
    return {
      [key]: entity,
      crumb,
    } as Record<TKey, TEntity> & { crumb: EntityCrumb }
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
