import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle, DetailedList } from '@green-ecolution/ui'
import { useSoilConditionLabel } from '@/hooks/details/useDetailsForSoilCondition'
import type { TreeCluster } from '@/api/backendApi'

interface ClusterMasterDataCardProps {
  treecluster: TreeCluster
}

const ClusterMasterDataCard = ({ treecluster }: ClusterMasterDataCardProps) => {
  const { t } = useTranslation(['treecluster', 'common'])
  const soilConditionLabel = useSoilConditionLabel()
  const species = [...new Set(treecluster.trees.map((tree) => tree.species))].join(', ')
  const noData = t('common:state.noData')

  const details = [
    { label: t('masterData.regionLabel'), value: treecluster.region?.name ?? noData },
    { label: t('masterData.speciesLabel'), value: species || noData },
    { label: t('masterData.soilLabel'), value: soilConditionLabel(treecluster.soilCondition) },
    { label: t('masterData.descriptionLabel'), value: treecluster.description || noData },
  ]

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('masterData.title')}</CardTitle>
      </CardHeader>
      <CardContent>
        <DetailedList details={details} columns={1} />
      </CardContent>
    </Card>
  )
}

export default ClusterMasterDataCard
