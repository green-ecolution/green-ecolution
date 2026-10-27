import { useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { PlanResponse } from '@green-ecolution/backend-client'
import EntityList from '@/components/general/EntityList'
import { clusterQueries } from '../../api/queries'
import TreeclusterCard from '../general/cards/TreeclusterCard'

const ClusterStopList = ({ plan }: { plan: PlanResponse }) => {
  const { t } = useTranslation('irrigation')
  const {
    data: { clusters },
  } = useSuspenseQuery(clusterQueries.stops(plan.stops))

  return (
    <EntityList
      items={clusters}
      getKey={(cluster) => cluster.id}
      emptyMessage={t('planning.stopList.emptyMessage')}
      renderItem={(cluster) => <TreeclusterCard treecluster={cluster} />}
    />
  )
}

export default ClusterStopList
