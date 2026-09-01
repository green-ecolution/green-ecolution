import { isMatch, useMatches } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'

export interface Breadcrumbs {
  title: string
  path: string
}

export function useBreadcrumbs(): Breadcrumbs[] {
  const matches = useMatches()
  const { t } = useTranslation('navigation')

  const breadcrumbs = matches
    .map((match) => {
      if (isMatch(match, 'loaderData.crumb') && match.loaderData?.crumb) {
        const crumb = match.loaderData.crumb
        const title = 'titleKey' in crumb ? t(`crumb.${crumb.titleKey}`) : crumb.title
        return {
          title,
          path: match.pathname,
        }
      }
    })
    .filter((b): b is Breadcrumbs => !!b)

  return breadcrumbs
}
