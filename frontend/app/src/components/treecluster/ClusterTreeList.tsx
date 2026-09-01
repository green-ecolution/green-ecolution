import {
  Badge,
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
import type { Tree } from '@/api/backendApi'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

interface ClusterTreeListProps {
  trees: Tree[]
}

const ClusterTreeList = ({ trees }: ClusterTreeListProps) => {
  const { t } = useTranslation('treecluster')

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('treeList.title', { count: trees.length })}</CardTitle>
      </CardHeader>
      <CardContent>
        {trees.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('treeList.emptyMessage')}</p>
        ) : (
          <div className="flex flex-col gap-y-3">
            {trees.map((tree) => {
              const statusDetails = getWateringStatusDetails(tree.wateringStatus)
              return (
                <ListCard key={tree.id} asChild size="compact">
                  <Link to="/trees/$treeId" params={{ treeId: tree.id }}>
                    <ListCardStatus status={statusDetails.color} />
                    <div className="min-w-0 flex-1">
                      <ListCardTitle>{tree.species}</ListCardTitle>
                      <ListCardDescription>{tree.number}</ListCardDescription>
                    </div>
                    {tree.sensor && (
                      <Badge variant="green-light">{t('treeList.sensorBadge')}</Badge>
                    )}
                  </Link>
                </ListCard>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterTreeList
