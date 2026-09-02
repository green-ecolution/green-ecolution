import { wateringPlanQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { getI18n } from '@/lib/i18n'
import { dateFnsLocale } from '@/lib/i18n/format'
import { createFileRoute } from '@tanstack/react-router'
import { format } from 'date-fns'

/** Shared by the detail and _formular layout routes; spread it — the router mutates route options on update. */
export const wateringPlanEntityRoute = entityRoute({
  key: 'wateringPlan',
  query: wateringPlanQueries.detail,
  idParam: 'wateringPlanId',
  title: (wateringPlan) => ({
    titleKey: 'wateringPlan:detail.entityTitle',
    params: {
      value: wateringPlan.date
        ? format(new Date(wateringPlan.date), 'dd.MM.yyyy', {
            locale: dateFnsLocale(getI18n().language),
          })
        : wateringPlan.id,
    },
  }),
  notFound: {
    entityName: { key: 'wateringPlan:entity.name' },
    backTo: '/watering-plans',
    backLabel: { key: 'wateringPlan:detail.notFoundBackLabel' },
  },
})

export const Route = createFileRoute('/_protected/watering-plans/$wateringPlanId')({
  ...wateringPlanEntityRoute,
})
