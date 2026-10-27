import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { cn } from '@green-ecolution/ui'
import { planningManifests } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'

interface PlanModuleTabsProps {
  current: string
}

const PlanModuleTabs = ({ current }: PlanModuleTabsProps) => {
  const { t, i18n } = useTranslation('wateringPlan')
  const { manifests } = useActiveModules()
  const planning = planningManifests(manifests)

  if (planning.length < 2) return null

  return (
    <nav
      aria-label={t('list.moduleTabsLabel')}
      className="mb-4 inline-flex rounded-xl bg-dark-100 p-1"
    >
      {planning.map(({ key, planning: { labelKey } }) => {
        const isCurrent = key === current
        return (
          <Link
            key={key}
            to="/plans"
            search={{ module: key }}
            aria-current={isCurrent ? 'page' : undefined}
            className={cn(
              'rounded-lg px-4 py-1.5 text-sm transition-colors',
              isCurrent
                ? 'bg-white font-semibold text-dark shadow-sm'
                : 'text-dark-600 hover:text-dark',
            )}
          >
            {i18n.t(labelKey)}
          </Link>
        )
      })}
    </nav>
  )
}

export default PlanModuleTabs
