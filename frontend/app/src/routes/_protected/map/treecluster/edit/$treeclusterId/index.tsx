import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useRef, useState } from 'react'
import { FormProvider, useWatch, type DefaultValues, type SubmitHandler } from 'react-hook-form'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { TreeResponse } from '@green-ecolution/backend-client'
import { clusterQueries, organizationQueries } from '@/api/queries'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import { entityNotFound, forbiddenErrorComponent, requirePermission } from '@/lib/router'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import FormForTreecluster from '@/components/general/form/FormForTreecluster'
import ForeignTreeDialog from '@/components/general/form/ForeignTreeDialog'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import { useTreeClusterForm } from '@/hooks/form/useTreeClusterForm'
import MapPanel from '@/components/map-gl/MapPanel'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { isMapAlive } from '@/components/map-gl/mapReady'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useSelectableTreeLayer, {
  type ForeignTree,
} from '@/components/map-gl/layers/useSelectableTreeLayer'

export const Route = createFileRoute('/_protected/map/treecluster/edit/$treeclusterId/')({
  component: EditClusterOnMap,
  beforeLoad: requirePermission(['tree_cluster:update']),
  loader: ({ context: { queryClient }, params: { treeclusterId } }) =>
    queryClient.prefetchQuery(clusterQueries.detail(treeclusterId)),
  errorComponent: forbiddenErrorComponent(
    entityNotFound({
      entityName: { key: 'map:clusterForm.entityName' },
      backTo: '/treecluster',
      backLabel: { key: 'map:clusterForm.backToList' },
    }),
  ),
})

function EditClusterOnMap() {
  const { t } = useTranslation('map')
  const { treeclusterId } = Route.useParams()
  const navigate = useNavigate({ from: Route.fullPath })
  const map = useMaplibreMap()
  const { data: cluster } = useSuspenseQuery(clusterQueries.detail(treeclusterId))

  // Frame the group once when the panel opens.
  const initialCluster = useRef(cluster)
  useEffect(() => {
    if (!isMapAlive(map)) return
    const framed = initialCluster.current
    const trees = framed.trees ?? []
    if (trees.length > 0) {
      const lngs = trees.map((t) => t.longitude)
      const lats = trees.map((t) => t.latitude)
      map.fitBounds(
        [
          [Math.min(...lngs), Math.min(...lats)],
          [Math.max(...lngs), Math.max(...lats)],
        ],
        { padding: 60, maxZoom: 17 },
      )
    } else if (framed.longitude !== 0 || framed.latitude !== 0) {
      map.flyTo({ center: [framed.longitude, framed.latitude], zoom: 17 })
    }
  }, [map])

  const initForm: DefaultValues<TreeclusterForm> = {
    name: cluster.name,
    address: cluster.address,
    description: cluster.description,
    soilCondition: cluster.soilCondition,
    treeIds: cluster.trees?.map((tree: TreeResponse) => tree.id) ?? [],
  }

  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useTreeClusterForm(
    'update',
    {
      clusterId: treeclusterId,
      initForm,
    },
  )
  const treeIds = useWatch({ control: form.control, name: 'treeIds' }) ?? []
  const [foreignTree, setForeignTree] = useState<ForeignTree | null>(null)
  const canReadOrganizations = useHasPermission(['organization:read'])
  // Only needed to name the organization in the dialog, so it waits for a click.
  const { data: organizations } = useQuery({
    ...organizationQueries.list(),
    enabled: canReadOrganizations && foreignTree !== null,
  })

  const toggleTree = useCallback(
    (id: string) => {
      const current = form.getValues('treeIds') ?? []
      const next = current.includes(id) ? current.filter((x) => x !== id) : [...current, id]
      form.setValue('treeIds', next, { shouldValidate: true, shouldDirty: true })
    },
    [form],
  )

  useClusterBoundaryLayer({ interactive: false })
  // The cluster's organization is fixed here, so only its own trees may join.
  // Foreign ones stay visible but dimmed; clicking one explains why.
  useSelectableTreeLayer({
    selectedIds: treeIds,
    onToggle: toggleTree,
    organizationId: cluster.organizationId,
    onForeignTree: setForeignTree,
  })

  const onSubmit: SubmitHandler<TreeclusterForm> = (data) => {
    mutate({ ...data, treeIds: data.treeIds ?? [] })
  }

  const handleCancel = () => {
    navigate({ to: '/map', search: (prev) => prev }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  return (
    <>
      <MapPanel title={t('clusterForm.editTitle')} onClose={handleCancel}>
        <p className="mb-5 shrink-0 text-sm text-dark-600">{t('clusterForm.clickTreesHint')}</p>
        <FormProvider {...form}>
          <FormForTreecluster
            displayError={isError}
            errorMessage={error?.message}
            onSubmit={onSubmit}
            onBlur={saveDraft}
            fullWidth
            emptyHint={t('clusterForm.emptyHint')}
          />
        </FormProvider>
      </MapPanel>

      <ForeignTreeDialog
        open={foreignTree !== null}
        onOpenChange={(open) => !open && setForeignTree(null)}
        organizationName={
          organizations?.find((org) => org.id === foreignTree?.organizationId)?.name
        }
        canSwitch={false}
        blockedReason={t('clusterForm.blockedReasonEdit')}
        selectedTreeCount={treeIds.length}
        onConfirm={() => setForeignTree(null)}
      />

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}
