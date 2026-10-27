import { createFileRoute } from '@tanstack/react-router'
import { WateringPlanStatus } from '@/api/backendApi'
import { DefaultValues, FormProvider } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import BackLink from '@/components/general/links/BackLink'
import { userQueries, vehicleQueries } from '@/api/queries'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import FormForWateringPlan from '@/components/general/form/FormForWateringPlan'
import { useWateringPlanForm } from '@/hooks/form/useWateringPlanForm'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import ModuleInactive from '@/modules/ModuleInactive'
import { planModule, planTextKey, type PlanningManifest } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'

export const Route = createFileRoute('/_protected/plans/_formular/new/')({
  component: NewWateringPlan,
})

const defaultForm = (module: string): DefaultValues<WateringPlanForm> => ({
  module,
  date: new Date(),
  description: '',
  transporterId: '',
  trailerId: undefined,
  transporterTankUsed: true,
  stopIds: [],
  status: WateringPlanStatus.Planned,
  driverIds: [],
  startPointName: '',
})

function NewWateringPlan() {
  const { manifests } = useActiveModules()
  const planManifest = planModule({}, manifests)
  return planManifest ? <NewWateringPlanForm planManifest={planManifest} /> : <ModuleInactive />
}

function NewWateringPlanForm({ planManifest }: { planManifest: PlanningManifest }) {
  const { key: module, planning } = planManifest
  const { t, i18n } = useTranslation('wateringPlan')
  const draft = useWateringPlanDraft<WateringPlanForm>('create')
  const initForm = draft.data ?? defaultForm(module)
  const formKey = draft.data?.stopIds?.join(',') ?? 'initial'

  const { data: users } = useSuspenseQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: trailers } = useSuspenseQuery(vehicleQueries.list())
  const { data: transporters } = useSuspenseQuery(vehicleQueries.list())
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useWateringPlanForm(
    'create',
    {
      initForm,
      planManifest,
    },
  )

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
          <FormForWateringPlan
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
