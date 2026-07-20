import { Card, CardContent, CardHeader, CardTitle, DetailedList } from '@green-ecolution/ui'
import { soilConditionLabel } from '@/hooks/details/useDetailsForSoilCondition'
import type { TreeCluster } from '@/api/backendApi'

interface ClusterMasterDataCardProps {
  treecluster: TreeCluster
}

const ClusterMasterDataCard = ({ treecluster }: ClusterMasterDataCardProps) => {
  const species = [...new Set(treecluster.trees.map((tree) => tree.species))].join(', ')

  const details = [
    { label: 'Region', value: treecluster.region?.name ?? 'Keine Angabe' },
    { label: 'Baumarten', value: species || 'Keine Angabe' },
    { label: 'Bodenart', value: soilConditionLabel(treecluster.soilCondition) },
    { label: 'Beschreibung', value: treecluster.description || 'Keine Angabe' },
  ]

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>Stammdaten</CardTitle>
      </CardHeader>
      <CardContent>
        <DetailedList details={details} columns={1} />
      </CardContent>
    </Card>
  )
}

export default ClusterMasterDataCard
