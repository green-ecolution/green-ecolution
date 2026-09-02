import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  ListCard,
  ListCardStatus,
  ListCardTitle,
  ListCardDescription,
} from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { Tree, TreeCluster } from '@/api/backendApi'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

interface TreeClusterCardProps {
  tree: Tree
  treeCluster?: TreeCluster
}

const TreeClusterCard = ({ tree, treeCluster }: TreeClusterCardProps) => {
  const { t } = useTranslation('tree')
  const getWateringStatusDetails = useWateringStatusDetails()
  const clusterStatus = treeCluster ? getWateringStatusDetails(treeCluster.wateringStatus) : null
  const treeStatus = getWateringStatusDetails(tree.wateringStatus)
  const deviates = treeCluster ? treeCluster.wateringStatus !== tree.wateringStatus : false
  const treeCount = treeCluster?.trees.length ?? 0

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('clusterCard.title')}</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {treeCluster && clusterStatus ? (
          <>
            <ListCard asChild size="compact">
              <Link to="/treecluster/$treeclusterId" params={{ treeclusterId: treeCluster.id }}>
                <ListCardStatus status={clusterStatus.color} />
                <div className="min-w-0 flex-1">
                  <ListCardTitle>{treeCluster.name}</ListCardTitle>
                  <ListCardDescription>
                    {clusterStatus.label} · {t('clusterCard.treeCount', { count: treeCount })} ·{' '}
                    {treeCluster.region?.name ?? t('clusterCard.noRegion')}
                  </ListCardDescription>
                </div>
              </Link>
            </ListCard>
            {deviates && (
              <p className="text-sm text-muted-foreground">
                {t('clusterCard.deviatesNote', { status: treeStatus.label })}
              </p>
            )}
          </>
        ) : (
          <p className="text-sm text-muted-foreground">{t('clusterCard.noClusterNote')}</p>
        )}
      </CardContent>
    </Card>
  )
}

export default TreeClusterCard
