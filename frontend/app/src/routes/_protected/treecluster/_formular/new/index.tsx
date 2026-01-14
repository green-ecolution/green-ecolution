import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { SoilCondition } from '@/api/backendApi'
import { DefaultValues, FormProvider, SubmitHandler } from 'react-hook-form'
import { clusterSchemaBase, TreeclusterForm } from '@/schema/treeclusterSchema'
import FormForTreecluster from '@/components/general/form/FormForTreecluster'
import useStore from '@/store/store'
import BackLink from '@/components/general/links/BackLink'
import { useTreeClusterForm } from '@/hooks/form/useTreeClusterForm'
import { safeJsonStorageParse } from '@/lib/utils'
import Modal from '@/components/general/Modal'

export const Route = createFileRoute('/_protected/treecluster/_formular/new/')({
  loader: () => {
    const { data } = safeJsonStorageParse('create-cluster', { schema: clusterSchemaBase })
    return {
      formState: data,
    }
  },
  component: NewTreecluster,
})

const defaultForm: DefaultValues<TreeclusterForm> = {
  soilCondition: SoilCondition.TreeSoilConditionUnknown,
  treeIds: [],
}

function NewTreecluster() {
  const { formState } = Route.useLoaderData()
  const { mutate, isError, error, form, navigationBlocker } = useTreeClusterForm('create', {
    initForm: formState ?? defaultForm,
  })
  const navigate = useNavigate({ from: Route.fullPath })

  const mapPosition = useStore((state) => ({
    lat: state.mapCenter[0],
    lng: state.mapCenter[1],
    zoom: state.mapZoom,
  }))

  const onSubmit: SubmitHandler<TreeclusterForm> = (data) => {
    mutate({
      ...data,
      treeIds: data.treeIds ?? [],
    })
  }

  const navigateToTreeSelect = () => {
    navigate({
      to: '/map/treecluster/select/tree',
      search: {
        lat: mapPosition.lat,
        lng: mapPosition.lng,
        zoom: mapPosition.zoom,
        formType: 'create',
        treeIds: form.getValues('treeIds'),
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <BackLink link={{ to: '/treecluster' }} label="Zu allen Bewässerungsgruppen" />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Neue Bewässerungsgruppe erstellen
        </h1>
        <p className="mb-5">
          In dieser Ansicht können Sie eine neue Bewässerungsgruppe erstellen sowie dieser Bäume
          zuweisen.
        </p>
      </article>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForTreecluster
            displayError={isError}
            onSubmit={onSubmit}
            onAddTrees={navigateToTreeSelect}
            errorMessage={error?.message}
          />
        </FormProvider>
      </section>

      <Modal
        title="Seite verlassen?"
        description={navigationBlocker.message}
        confirmText="Verlassen"
        isOpen={navigationBlocker.isModalOpen}
        onConfirm={navigationBlocker.confirmLeave}
        onCancel={navigationBlocker.closeModal}
      />
    </div>
  )
}

export default NewTreecluster
