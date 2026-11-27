import { createFileRoute } from '@tanstack/react-router'
import { VehicleType, PlanStatus } from '@/api/backendApi'
import { DefaultValues, FormProvider } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import { z } from 'zod'
import { useTranslation } from 'react-i18next'
import BackLink from '@/components/general/links/BackLink'
import { userQueries, vehicleQueries } from '@/api/queries'
import { PlanForm } from '@/schema/planSchema'
import FormForPlan from '@/components/general/form/FormForPlan'
import { usePlanForm } from '@/hooks/form/usePlanForm'
import { usePlanDraft } from '@/store/form/useFormDraft'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import ModuleInactive from '@/modules/ModuleInactive'
import { draftForModule, planModule, planTextKey, type PlanningManifest } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'

export const Route = createFileRoute('/_protected/plans/_formular/new/')({
  component: NewPlan,
  validateSearch: z.object({ module: z.string().optional().catch(undefined) }),
})

const defaultForm = (module: string): DefaultValues<PlanForm> => ({
  module,
  date: new Date(),
  description: '',
  transporterId: '',
  trailerId: undefined,
  transporterTankUsed: true,
  stopIds: [],
  status: PlanStatus.Planned,
  driverIds: [],
  startPointName: '',
})

function NewPlan() {
  const { manifests } = useActiveModules()
  const search = Route.useSearch()
  const draft = usePlanDraft<PlanForm>('create')
  const planManifest = planModule({ module: search.module ?? draft.data?.module }, manifests)
  return planManifest ? <NewPlanForm planManifest={planManifest} /> : <ModuleInactive />
}

function NewPlanForm({ planManifest }: { planManifest: PlanningManifest }) {
  const { key: module, planning } = planManifest
  const { t, i18n } = useTranslation('wateringPlan')
  const draft = usePlanDraft<PlanForm>('create')
  const moduleDraft = draftForModule(draft.data, module)
  const initForm = moduleDraft ?? defaultForm(module)
  const formKey = `${module}:${moduleDraft?.stopIds?.join(',') ?? 'initial'}`

  const { data: users } = useSuspenseQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: trailers } = useSuspenseQuery(vehicleQueries.list({ type: [VehicleType.Trailer] }))
  const { data: transporters } = useSuspenseQuery(
    vehicleQueries.list({ type: [VehicleType.Transporter] }),
  )
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = usePlanForm('create', {
    initForm,
    planManifest,
  })

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <BackLink link={{ to: '/plans' }} label={t('create.backLabel')} />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('create.title')}
        </h1>
        <p className="mb-5">
          {i18n.t(planTextKey(planning, 'createDescription', 'wateringPlan:create.description'))}
        </p>
      </article>

      <section className="mt-10">
        <FormProvider key={formKey} {...form}>
          <FormForPlan
            displayError={isError}
            onSubmit={(data) => mutate(data)}
            trailers={trailers.data}
            transporters={transporters.data}
            users={users.data}
            formType="create"
            saveDraft={saveDraft}
            errorMessage={error?.message}
            onBlur={saveDraft}
          />
        </FormProvider>
      </section>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </div>
  )
}
