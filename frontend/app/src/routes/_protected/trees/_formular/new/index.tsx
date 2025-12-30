import FormForTree from '@/components/general/form/FormForTree'
import { TreeForm, treeSchemaBase } from '@/schema/treeSchema'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useMapStore } from '@/store/store'
import { sensorQuery, treeClusterQuery } from '@/api/queries'
import { useTreeForm } from '@/hooks/form/useTreeForm'
import { useCallback } from 'react'
import { z } from 'zod'
import { safeJsonStorageParse } from '@/lib/utils'
import { FormProvider } from 'react-hook-form'
import Modal from '@/components/general/Modal'

const newTreeSearchSchema = z.object({
  lat: z.number(),
  lng: z.number(),
})

export const Route = createFileRoute('/_protected/trees/_formular/new/')({
  component: NewTree,
  validateSearch: newTreeSearchSchema,
  loaderDeps: ({ search }) => {
    return search
  },
  loader: ({ context: { queryClient }, deps }) => {
    queryClient
      .prefetchQuery(sensorQuery())
      .catch((error) => console.error('Prefetching "sensorQuery" failed:', error))
    queryClient
      .prefetchQuery(treeClusterQuery())
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))

    const { data, success } = safeJsonStorageParse('create-tree', { schema: treeSchemaBase })
    return {
      lat: success ? data.latitude : deps.lat,
      lng: success ? data.longitude : deps.lng,
      formState: data,
    }
  },
})

const defaultForm = (lat: number, lng: number) => ({
  latitude: lat,
  longitude: lng,
  plantingYear: new Date().getFullYear(),
  treeClusterId: -1,
  sensorId: '-1',
})

function NewTree() {
  const { lat, lng, formState } = Route.useLoaderData()
  const { mutate, isError, error, form, navigationBlocker } = useTreeForm('create', {
    initForm: formState ?? defaultForm(lat, lng),
  })
  const navigate = useNavigate({ from: Route.fullPath })
  const map = useMapStore()
  const { data: sensors } = useSuspenseQuery(sensorQuery())
  const { data: treeClusters } = useSuspenseQuery(treeClusterQuery())

  const onSubmit = (data: TreeForm) => {
    mutate({
      ...data,
      sensorId: data.sensorId && data.sensorId !== '-1' ? data.sensorId : undefined,
      treeClusterId: data.treeClusterId && data.treeClusterId <= 0 ? undefined : data.treeClusterId,
    })
  }

  const handleOnChangeLocation = useCallback(() => {
    navigate({
      to: '/map/tree/edit',
      search: {
        lat: form.getValues('latitude'),
        lng: form.getValues('longitude'),
        treeLat: form.getValues('latitude'),
        treeLng: form.getValues('longitude'),
        formType: 'create',
        zoom: map.zoom,
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }, [form, map.zoom, navigate])

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Neuen Baum erfassen
        </h1>
        <p className="mb-5">
          Hier können Sie einen neuen Baum erstellen. Dieser wird im System als "manuell erstellt"
          erfasst.
        </p>
      </article>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForTree
            isReadonly={false}
            treeClusters={treeClusters.data}
            sensors={sensors.data}
            displayError={isError}
            onChangeLocation={handleOnChangeLocation}
            errorMessage={error?.message}
            onSubmit={onSubmit}
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
