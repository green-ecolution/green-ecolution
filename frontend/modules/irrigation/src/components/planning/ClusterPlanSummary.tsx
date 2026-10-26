import { StatusCard } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import { PlanStatus, type PlanResponse } from '@green-ecolution/backend-client'

const ClusterPlanSummary = ({ plan }: { plan: PlanResponse }) => {
  const { t } = useTranslation('irrigation')
  if (plan.status !== PlanStatus.Finished) return null

  const consumed = plan.outcomes
    .filter(({ completed }) => completed)
    .reduce((sum, { amount }) => sum + (amount ?? 0), 0)

  return (
    <li>
      <StatusCard
        label={t('planning.summary.consumedWaterLabel')}
        value={t('planning.summary.consumedWaterValue', { value: consumed })}
        isLarge
        description={t('planning.summary.consumedWaterDescription', { count: plan.stops.length })}
      />
    </li>
  )
}

export default ClusterPlanSummary
