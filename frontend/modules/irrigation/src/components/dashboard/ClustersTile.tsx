import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { LinkCard, LinkCardDescription, LinkCardFooter, LinkCardTitle } from '@green-ecolution/ui'
import type { DashboardTileProps } from '@/modules/types'

// Typed as string like the other dashboard cards, which link without search params.
const clustersUrl = (): string => '/treecluster'

const ClustersTile = ({ variant }: DashboardTileProps) => {
  const { t } = useTranslation('irrigation')
  const linkLabel = t('dashboard.clusters.linkLabel')

  return (
    <li>
      <LinkCard variant={variant} asChild>
        <Link to={clustersUrl()} aria-label={linkLabel}>
          <LinkCardTitle>{t('dashboard.clusters.headline')}</LinkCardTitle>
          <LinkCardDescription>{t('dashboard.clusters.description')}</LinkCardDescription>
          <LinkCardFooter>{linkLabel}</LinkCardFooter>
        </Link>
      </LinkCard>
    </li>
  )
}

export default ClustersTile
