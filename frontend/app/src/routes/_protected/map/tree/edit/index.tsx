import { DragableMarker } from '@/components/map/MapMarker'
import MapSelectEntitiesModal from '@/components/map/MapSelectEntitiesModal'
import { safeJsonStorageParse } from '@/lib/utils'
import { TreeForm, treeSchemaBase } from '@/schema/treeSchema'
import { useMapStore } from '@/store/store'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { LatLng } from 'leaflet'
import { useCallback, useState } from 'react'
import { z } from 'zod'

const editTreeParams = z.object({
  treeLat: z.number(),
  treeLng: z.number(),
  treeId: z.number().optional(),
  formType: z.enum(['create', 'update']),
})

export const Route = createFileRoute('/_protected/map/tree/edit/')({
  component: EditTree,
  validateSearch: editTreeParams,
  loaderDeps: ({ search: { treeLat, treeLng, treeId, formType } }) => ({
    treeLat,
    treeLng,
    treeId,
    formType,
  }),
})

function EditTree() {
  const navigate = useNavigate({ from: Route.fullPath })
  const { treeId, treeLat, treeLng, formType } = Route.useSearch()
  const { zoom } = useMapStore()
  const [treeLatLng, setTreeLatLng] = useState<LatLng>(() => new LatLng(treeLat, treeLng))

  const handleNavigateBack = useCallback(() => {
    switch (formType) {
      case 'create':
        return navigate({
          to: '/trees/new',
          search: { lat: treeLatLng.lat, lng: treeLatLng.lng },
        })
      case 'update':
        if (treeId) {
          return navigate({
            to: `/trees/$treeId/edit`,
            params: { treeId: treeId.toString() },
          })
        } else {
          throw new Error('treeId is undefined in update tree step')
        }
      default:
        return navigate({
          to: '/map',
          search: { lat: treeLatLng.lat, lng: treeLatLng.lng, zoom },
        })
    }
  }, [formType, navigate, treeLatLng.lat, treeLatLng.lng, treeId, zoom])

  const handleSave = () => {
    const { data, success, error } = safeJsonStorageParse<TreeForm>(`${formType}-tree`, {
      schema: treeSchemaBase,
    })
    if (success) {
      data.latitude = treeLatLng.lat
      data.longitude = treeLatLng.lng
      window.sessionStorage.setItem(`${formType}-tree`, JSON.stringify(data))
    } else {
      console.error(error)
    }
    handleNavigateBack().catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <>
      <MapSelectEntitiesModal
        onSave={handleSave}
        onCancel={() => void handleNavigateBack()}
        title="Baum erfassen:"
        content={
          <ul className="space-y-3">
            <li className="text-dark-600">
              {treeLatLng ? (
                <>
                  <p>Neuer Baum an folgendem Standort:</p>
                  {treeLatLng.lat}, {treeLatLng.lng}
                </>
              ) : (
                <p>Bitte wähle einen Standort für den neuen Baum.</p>
              )}
            </li>
          </ul>
        }
      />

      {treeLatLng && (
        <DragableMarker position={treeLatLng} onMove={(latlng) => setTreeLatLng(latlng)} />
      )}
    </>
  )
}
