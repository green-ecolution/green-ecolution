import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { FormProvider } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import type { WasteBinForm } from '@green-ecolution/domain-wasm'
import FormPageHeader from '@/components/general/FormPageHeader'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { pendingLoading } from '@/lib/router'
import { wasteBinQueries } from '../../../../../../api/queries'
import type { WasteBin } from '../../../../../../api/client'
import FormForWasteBin from '../../../../../../components/FormForWasteBin'
import { useWasteBinForm } from '../../../../../../hooks/useWasteBinForm'

const wasteBinFormRoute = getRouteApi('/_protected/waste-bins/_formular/$wasteBinId')

export const Route = createFileRoute('/_protected/waste-bins/_formular/$wasteBinId/edit/')({
  component: EditWasteBin,
  pendingComponent: pendingLoading({ key: 'waste:detail.loadingLabel' }),
})

function EditWasteBin() {
  const { t } = useTranslation('waste')
  const { wasteBinId } = wasteBinFormRoute.useParams()
  const { initForm, loadedData } = useInitFormQuery<WasteBin, WasteBinForm>(
    wasteBinQueries.detail(wasteBinId),
    (bin) => ({
      number: bin.number,
      latitude: bin.latitude,
      longitude: bin.longitude,
      volumeLiters: bin.volumeLiters,
      depthCm: bin.depthCm,
    }),
  )
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useWasteBinForm('update', {
    wasteBinId,
    initForm,
  })

  const onSubmit = (data: WasteBinForm) => mutate(data)

  return (
    <div className="container mt-6">
      <FormPageHeader
        backLink={{
          link: { to: '/waste-bins/$wasteBinId', params: { wasteBinId } },
          label: t('update.backLabel'),
        }}
        title={t('update.title', { number: loadedData.number })}
      >
        <p className="mb-5">{t('update.description')}</p>
      </FormPageHeader>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForWasteBin
            displayError={isError}
            errorMessage={error?.message}
            onSubmit={onSubmit}
            onBlur={saveDraft}
          />
        </FormProvider>
      </section>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </div>
  )
}
