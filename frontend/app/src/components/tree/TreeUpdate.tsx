import { TreeForm, treeSchemaBase } from '@/schema/treeSchema'
import FormForTree from '../general/form/FormForTree'
import BackLink from '../general/links/BackLink'
import DeleteSection from '../treecluster/DeleteSection'
import { Sensor, Tree, TreeCluster } from '@green-ecolution/backend-client'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { treeIdQuery } from '@/api/queries'
import { treeApi } from '@/api/backendApi'
import { useMapStore } from '@/store/store'
import { useNavigate } from '@tanstack/react-router'
import { useTreeForm } from '@/hooks/form/useTreeForm'
import { safeJsonStorageParse } from '@/lib/utils'
import { FormProvider } from 'react-hook-form'
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

interface TreeUpdateProps {
  treeId: string
  clusters: TreeCluster[]
  sensors: Sensor[]
}

const TreeUpdate = ({ treeId, clusters, sensors }: TreeUpdateProps) => {
  const navigate = useNavigate()
  const map = useMapStore()
  const { data: formState } = safeJsonStorageParse('update-tree', { schema: treeSchemaBase })
  const { initForm, loadedData } = useInitFormQuery<Tree, TreeForm>(
    treeIdQuery(treeId),
    (data) =>
      formState ?? {
        latitude: data.latitude,
        longitude: data.longitude,
        number: data.number,
        species: data.species,
        plantingYear: data.plantingYear,
        treeClusterId: data.treeClusterId ?? -1,
        sensorId: data.sensor?.id ?? '-1',
        description: data.description,
        provider: data.provider,
      },
  )
  const { mutate, isError, error, form, navigationBlocker } = useTreeForm('update', {
    initForm,
    treeId,
  })

  const onSubmit = (data: TreeForm) => {
    mutate({
      ...data,
      sensorId: data.sensorId && data.sensorId === '-1' ? undefined : data.sensorId,
      treeClusterId: data.treeClusterId && data.treeClusterId <= 0 ? undefined : data.treeClusterId,
    })
  }

  const handleDeleteTree = () => {
    return treeApi.deleteTree({
      treeId: Number(treeId),
    })
  }

  const handleOnChangeLocation = () => {
    navigate({
      to: '/map/tree/edit',
      search: {
        treeId: Number(treeId),
        lat: form.getValues('latitude'),
        lng: form.getValues('longitude'),
        treeLat: form.getValues('latitude'),
        treeLng: form.getValues('longitude'),
        formType: 'update',
        zoom: map.mapZoom,
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <>
      <BackLink link={{ to: '/trees/$treeId', params: { treeId } }} label="Zurück zur Übersicht" />
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Baum {loadedData.number} bearbeiten
        </h1>
        <p className="mb-5">In dieser Ansicht können Sie einem Baum bearbeiten.</p>
      </article>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForTree
            isReadonly={!!initForm?.provider}
            displayError={isError}
            onSubmit={onSubmit}
            treeClusters={clusters}
            sensors={sensors}
            onChangeLocation={handleOnChangeLocation}
            errorMessage={error?.message}
          />
        </FormProvider>
      </section>

      {!initForm?.provider && (
        <DeleteSection
          mutationFn={handleDeleteTree}
          entityName="der Baum"
          redirectUrl={{ to: '/map' }}
        />
      )}

      <AlertDialog open={navigationBlocker.isModalOpen} onOpenChange={(open) => !open && navigationBlocker.closeModal()}>
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
              <MoveRight />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

export default TreeUpdate
