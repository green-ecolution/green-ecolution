import { isMatch, useMatches } from '@tanstack/react-router'

export interface Breadcrumbs {
  title: string
  path: string
}

export function useBreadcrumbs(): Breadcrumbs[] {
  const matches = useMatches()

  const breadcrumbs = matches
    .map((match) => {
      if (isMatch(match, 'loaderData.crumb') && match.loaderData?.crumb) {
        return {
          title: match.loaderData.crumb.title,
          path: match.pathname,
        }
      }
    })
    .filter((b): b is Breadcrumbs => !!b)

  return breadcrumbs
}
