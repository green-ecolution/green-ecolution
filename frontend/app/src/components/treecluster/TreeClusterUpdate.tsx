import FormForTreecluster from '../general/form/FormForTreecluster'
import BackLink from '../general/links/BackLink'
import DeleteSection from './DeleteSection'
import { TreeCluster } from '@green-ecolution/backend-client'
import { clusterSchemaBase, TreeclusterForm } from '@/schema/treeclusterSchema'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { treeClusterIdQuery } from '@/api/queries'
import { clusterApi } from '@/api/backendApi'
import { useNavigate } from '@tanstack/react-router'
import { FormProvider, SubmitHandler } from 'react-hook-form'
import useStore from '@/store/store'
import { Suspense } from 'react'
import LoadingInfo from '../general/error/LoadingInfo'
import { useTreeClusterForm } from '@/hooks/form/useTreeClusterForm'
import { safeJsonStorageParse } from '@/lib/utils'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'

interface TreeClusterUpdateProps {
  clusterId: string
}

const TreeClusterUpdate = ({ clusterId }: TreeClusterUpdateProps) => {
  const mapCenter = useStore((state) => state.mapCenter)
  const mapZoom = useStore((state) => state.mapZoom)
  const mapPosition = { lat: mapCenter[0], lng: mapCenter[1], zoom: mapZoom }
  const navigate = useNavigate()
  const { data: formState } = safeJsonStorageParse('update-cluster', { schema: clusterSchemaBase })
  const { initForm, loadedData } = useInitFormQuery<TreeCluster, TreeclusterForm>(
    treeClusterIdQuery(clusterId),
    (data) =>
      formState ?? {
        name: data.name,
        address: data.address,
        description: data.description,
        soilCondition: data.soilCondition,
        treeIds: data.trees?.map((tree) => tree.id) ?? [],
      },
  )
  const { mutate, isError, error, form, navigationBlocker } = useTreeClusterForm('update', {
    clusterId,
    initForm,
  })

  const onSubmit: SubmitHandler<TreeclusterForm> = (data) => {
    mutate({
      ...data,
      treeIds: data.treeIds,
    })
  }

  const handleDeleteTreeCluster = () => {
    return clusterApi.deleteTreeCluster({
      clusterId: Number(clusterId),
    })
  }

  const navigateToTreeSelect = () => {
    navigate({
      to: '/map/treecluster/select/tree',
      search: {
        lat: mapPosition.lat,
        lng: mapPosition.lng,
        zoom: mapPosition.zoom,
        clusterId: Number(clusterId),
        treeIds: form.getValues('treeIds'),
        formType: 'update',
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <>
      <article className="2xl:w-4/5">
        <BackLink
          label="Zurück zur Bewässerungsgruppe"
          link={{
            to: `/treecluster/$treeclusterId`,
            params: { treeclusterId: clusterId?.toString() ?? '' },
          }}
        />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Bewässerungsgruppe {loadedData?.name} bearbeiten
        </h1>
        <p className="mb-5">
          Hier können Sie Bäume der aktuell ausgewählten Bewässerungsgruppe zuweisen oder entfernen
          sowie auch Name und Adresse angeben.
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

      <Suspense fallback={<LoadingInfo label="Die Bewässerungsgruppe wird gelöscht" />}>
        <DeleteSection
          mutationFn={handleDeleteTreeCluster}
          entityName="die Bewässerungsgruppe"
          redirectUrl={{ to: '/treecluster' }}
        />
      </Suspense>

      <AlertDialog
        open={navigationBlocker.isModalOpen}
        onOpenChange={(open) => !open && navigationBlocker.closeModal()}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Seite verlassen?</AlertDialogTitle>
            <AlertDialogDescription>{navigationBlocker.message}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={navigationBlocker.closeModal}>
              Abbrechen
              <X />
            </AlertDialogCancel>
            <AlertDialogAction onClick={navigationBlocker.confirmLeave}>
              Verlassen
              <MoveRight className="icon-arrow-animate" />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

export default TreeClusterUpdate
