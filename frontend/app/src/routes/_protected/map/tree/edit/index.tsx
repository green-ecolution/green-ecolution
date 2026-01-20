import { DragableMarker } from '@/components/map/MapMarker'
import MapSelectEntitiesModal from '@/components/map/MapSelectEntitiesModal'
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
import { useTreeDraft } from '@/store/form/useFormDraft'
import { TreeForm } from '@/schema/treeSchema'
import { useMapStore } from '@/store/store'
import { createFileRoute, useNavigate, useBlocker } from '@tanstack/react-router'
import { LatLng } from 'leaflet'
import { useCallback, useRef, useState } from 'react'
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
  const { mapZoom: zoom } = useMapStore()
  const [treeLatLng, setTreeLatLng] = useState<LatLng>(() => new LatLng(treeLat, treeLng))
  const allowNavigationRef = useRef(false)
  const draft = useTreeDraft<TreeForm>(formType)

  const { proceed, reset, status } = useBlocker({
    shouldBlockFn: ({ next }) => {
      if (allowNavigationRef.current) {
        allowNavigationRef.current = false
        return false
      }

      const isAllowedPath =
        next.pathname.startsWith('/trees/new') ||
        next.pathname.startsWith('/trees/') ||
        next.pathname.startsWith('/map/tree/edit')
      if (isAllowedPath) {
        return false
      }

      return true
    },
    withResolver: true,
  })

  const handleConfirmLeave = useCallback(() => {
    draft.clear()
    proceed?.()
  }, [proceed, draft])

  const handleNavigateBack = useCallback(() => {
    allowNavigationRef.current = true
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
    const currentData = draft.data
    if (currentData) {
      const coordsChanged =
        currentData.latitude !== treeLatLng.lat || currentData.longitude !== treeLatLng.lng

      draft.updateData((prev) => ({
        ...(prev ?? ({} as TreeForm)),
        latitude: treeLatLng.lat,
        longitude: treeLatLng.lng,
      }))

      if (coordsChanged) {
        draft.markChanged()
      }
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
          <ul className="flex flex-col gap-y-3">
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

      <AlertDialog open={status === 'blocked'} onOpenChange={(open) => !open && reset?.()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Seite verlassen?</AlertDialogTitle>
            <AlertDialogDescription>
              Möchtest du die Seite wirklich verlassen? Deine Eingaben gehen verloren, wenn du jetzt
              gehst.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => reset?.()}>
              Abbrechen
              <X />
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmLeave}>
              Verlassen
              <MoveRight className="icon-arrow-animate" />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
