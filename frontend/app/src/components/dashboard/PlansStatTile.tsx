import { useQueries } from '@tanstack/react-query'
import { isToday } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { PlanStatus } from '@green-ecolution/backend-client'
import { wateringPlanQueries } from '@/api/queries'
import { usePermissions } from '@/lib/auth/usePermissions'
import { satisfies } from '@/lib/auth/permissions'
import { planningManifests } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'
import DashboardStatTile from './DashboardStatTile'

/** Running plans across every planning module the user may read, plus those planned for today. */
const PlansStatTile = () => {
  const { t } = useTranslation('dashboard')
  const { manifests } = useActiveModules()
  const permissions = usePermissions()
  const modules = planningManifests(manifests)
    .filter(({ planning }) => satisfies(permissions, [`${planning.planResource}:read`]))
    .map(({ key }) => key)

  // Same keys as the board columns, so opening the board after this reuses the cache.
  const active = useQueries({
    queries: modules.map((module) => wateringPlanQueries.boardColumn(module, [PlanStatus.Active])),
  })
  const planned = useQueries({
    queries: modules.map((module) => wateringPlanQueries.boardColumn(module, [PlanStatus.Planned])),
  })
  if (modules.length === 0) return null

  const loaded = [...active, ...planned].every(({ data }) => data !== undefined)
  const activeCount = active.reduce((sum, { data }) => sum + (data?.data.length ?? 0), 0)
  const todayCount = planned.reduce(
    (sum, { data }) =>
      sum + (data?.data.filter((plan) => isToday(new Date(plan.date))).length ?? 0),
    0,
  )

  return (
    <DashboardStatTile
      label={t('overview.plans.label')}
      value={loaded ? activeCount : undefined}
      status={activeCount > 0 ? 'green-dark' : 'default'}
      description={loaded ? t('overview.plans.description', { count: todayCount }) : undefined}
      link={{ to: '/plans' }}
    />
  )
}

export default PlansStatTile
