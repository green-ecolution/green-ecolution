import { StatusCard } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import { PlanStatus, type PlanResponse } from '@green-ecolution/backend-client'

const BinPlanSummary = ({ plan }: { plan: PlanResponse }) => {
  const { t } = useTranslation('waste')
  if (plan.status !== PlanStatus.Finished) return null

  const emptied = plan.outcomes.filter(({ completed }) => completed)
  const reported = emptied.filter(({ amount }) => amount != null)
  const collected = reported.reduce((sum, { amount }) => sum + (amount ?? 0), 0)

  return (
    <li>
      <StatusCard
        label={t('planning.summary.emptiedLabel')}
        value={t('planning.summary.emptiedValue', {
          emptied: emptied.length,
          total: plan.stops.length,
        })}
        isLarge
        description={
          reported.length > 0
            ? t('planning.summary.collectedDescription', { value: collected })
            : t('planning.summary.emptiedDescription')
        }
      />
    </li>
  )
}

export default BinPlanSummary
