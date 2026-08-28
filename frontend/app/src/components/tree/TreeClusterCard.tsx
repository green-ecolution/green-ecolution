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
import type { Tree, TreeCluster } from '@/api/backendApi'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

interface TreeClusterCardProps {
  tree: Tree
  treeCluster?: TreeCluster
}

const TreeClusterCard = ({ tree, treeCluster }: TreeClusterCardProps) => {
  const clusterStatus = treeCluster ? getWateringStatusDetails(treeCluster.wateringStatus) : null
  const treeStatus = getWateringStatusDetails(tree.wateringStatus)
  const deviates = treeCluster ? treeCluster.wateringStatus !== tree.wateringStatus : false
  const treeCount = treeCluster?.trees.length ?? 0

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>Bewässerungsgruppe</CardTitle>
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
                    {clusterStatus.label} · {treeCount} {treeCount === 1 ? 'Baum' : 'Bäume'} ·{' '}
                    {treeCluster.region?.name ?? 'Keine Region'}
                  </ListCardDescription>
                </div>
              </Link>
            </ListCard>
            {deviates && (
              <p className="text-sm text-muted-foreground">
                Der Gruppenzustand ist ein Mehrheitswert. Dieser Baum ist als »{treeStatus.label}«
                bewertet.
              </p>
            )}
          </>
        ) : (
          <p className="text-sm text-muted-foreground">
            Dieser Baum ist keiner Bewässerungsgruppe zugeordnet. Ohne Gruppe fehlt die Bodenart,
            weshalb sich der Bewässerungszustand aus Feuchtemesswerten nicht berechnen lässt.
          </p>
        )}
      </CardContent>
    </Card>
  )
}

export default TreeClusterCard
