import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { StatusCard } from '@green-ecolution/ui'
import StatusCardGrid from '@/components/general/StatusCardGrid'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import type { Tree } from '@/api/backendApi'

interface TreeKpiRowProps {
  tree: Tree
}

const TreeKpiRow = ({ tree }: TreeKpiRowProps) => {
  const { t } = useTranslation('tree')
  const dateLocale = useDateLocale()
  const getWateringStatusDetails = useWateringStatusDetails()
  const wateringStatus = getWateringStatusDetails(tree.wateringStatus)

  return (
    <StatusCardGrid className="lg:grid-cols-2">
      <li className="h-full">
        <StatusCard
          size="compact"
          status={wateringStatus.color}
          indicator="dot"
          label={t('kpi.wateringStatusLabel')}
          value={wateringStatus.label}
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label={t('kpi.lastWateredLabel')}
          value={
            tree.lastWatered
              ? format(new Date(tree.lastWatered), 'dd.MM.yyyy', { locale: dateLocale })
              : '—'
          }
          info={t('kpi.lastWateredInfo')}
        />
      </li>
    </StatusCardGrid>
  )
}

export default TreeKpiRow
