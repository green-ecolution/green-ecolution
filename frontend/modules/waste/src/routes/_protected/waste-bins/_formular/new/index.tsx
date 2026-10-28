import { createFileRoute } from '@tanstack/react-router'
import { FormProvider } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import type { WasteBinForm } from '@green-ecolution/domain-wasm'
import FormPageHeader from '@/components/general/FormPageHeader'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import FormForWasteBin from '../../../../../components/FormForWasteBin'
import { useWasteBinForm, useWasteBinOrganizations } from '../../../../../hooks/useWasteBinForm'

export const Route = createFileRoute('/_protected/waste-bins/_formular/new/')({
  component: NewWasteBin,
})

function NewWasteBin() {
  const { t } = useTranslation('waste')
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useWasteBinForm('create', {
    initForm: { number: '' },
  })
  const organizations = useWasteBinOrganizations(form)

  const onSubmit = (data: WasteBinForm) => mutate(data)

  return (
    <div className="container mt-6">
      <FormPageHeader
        backLink={{ link: { to: '/waste-bins' }, label: t('create.backLabel') }}
        title={t('create.title')}
      >
        <p className="mb-5">{t('create.description')}</p>
      </FormPageHeader>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForWasteBin
            organizations={organizations}
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
