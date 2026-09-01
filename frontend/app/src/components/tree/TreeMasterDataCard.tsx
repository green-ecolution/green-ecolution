import { Card, CardContent, CardHeader, CardTitle, DetailedList } from '@green-ecolution/ui'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import type { Tree } from '@/api/backendApi'

interface TreeMasterDataCardProps {
  tree: Tree
}

const TreeMasterDataCard = ({ tree }: TreeMasterDataCardProps) => {
  const { t } = useTranslation(['tree', 'common'])
  const dateLocale = useDateLocale()
  const noData = t('common:state.noData')

  const details = [
    { label: t('masterData.speciesLabel'), value: tree.species || noData },
    { label: t('masterData.numberLabel'), value: tree.number || noData },
    { label: t('masterData.plantingYearLabel'), value: `${tree.plantingYear}` },
    {
      label: t('masterData.dataSourceLabel'),
      value: tree.provider ?? t('masterData.manuallyCreated'),
    },
    {
      label: t('masterData.coordinatesLabel'),
      value: `${tree.latitude.toFixed(6)}, ${tree.longitude.toFixed(6)}`,
    },
    {
      label: t('masterData.lastUpdateLabel'),
      value: format(new Date(tree.updatedAt), 'dd.MM.yyyy', { locale: dateLocale }),
    },
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

export default TreeMasterDataCard
