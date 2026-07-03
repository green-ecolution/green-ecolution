import { wateringPlanIdQuery } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'
import { format } from 'date-fns'

/** Shared by the detail and _formular layout routes; spread it — the router mutates route options on update. */
export const wateringPlanEntityRoute = entityRoute({
  key: 'wateringPlan',
  query: wateringPlanIdQuery,
  idParam: 'wateringPlanId',
  title: (wateringPlan) =>
    wateringPlan.date
      ? `Einsatz: ${format(new Date(wateringPlan.date), 'dd.MM.yyyy')}`
      : `Einsatz: ${wateringPlan.id}`,
  notFound: {
    entityName: 'Einsatzplan',
    backTo: '/watering-plans',
    backLabel: 'Zur Einsatzliste',
  },
})

export const Route = createFileRoute('/_protected/watering-plans/$wateringPlanId')({
  ...wateringPlanEntityRoute,
})
