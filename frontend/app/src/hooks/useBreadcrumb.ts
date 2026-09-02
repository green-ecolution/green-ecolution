import { isMatch, useMatches } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'

export interface Breadcrumbs {
  title: string
  path: string
}

export function useBreadcrumbs(): Breadcrumbs[] {
  const matches = useMatches()
  const { t, i18n } = useTranslation('navigation')

  const breadcrumbs = matches
    .map((match) => {
      if (isMatch(match, 'loaderData.crumb') && match.loaderData?.crumb) {
        const crumb = match.loaderData.crumb
        // Entity crumbs carry a fully-qualified cross-namespace key, so they
        // resolve via the instance directly rather than the navigation-scoped `t`.
        const title =
          'title' in crumb
            ? crumb.title
            : 'params' in crumb
              ? i18n.t(crumb.titleKey, crumb.params)
              : t(`crumb.${crumb.titleKey}`)
        return {
          title,
          path: match.pathname,
        }
      }
    })
    .filter((b): b is Breadcrumbs => !!b)

  return breadcrumbs
}
