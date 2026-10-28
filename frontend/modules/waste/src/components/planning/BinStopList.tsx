import { useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { PlanResponse } from '@green-ecolution/backend-client'
import EntityList from '@/components/general/EntityList'
import { wasteBinQueries } from '../../api/queries'
import WasteBinCard from '../WasteBinCard'

const BinStopList = ({ plan }: { plan: PlanResponse }) => {
  const { t } = useTranslation('waste')
  const { data: bins } = useSuspenseQuery(wasteBinQueries.byIds(plan.stops))

  return (
    <EntityList
      items={bins}
      getKey={(bin) => bin.id}
      emptyMessage={t('planning.stopList.emptyMessage')}
      renderItem={(bin) => <WasteBinCard bin={bin} />}
    />
  )
}

export default BinStopList
