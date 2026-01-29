import { treeApi } from '@/api/backendApi'
import { sensorIdQuery, treeIdQuery } from '@/api/queries'
import SelectedCard from '@/components/general/cards/SelectedCard'
import MapSelectEntitiesModal from '@/components/map/MapSelectEntitiesModal'
import SensorMarker from '@/components/map/marker/SensorMarker'
import WithFilterableTrees from '@/components/map/marker/WithFilterableTrees'
import createToast from '@/hooks/createToast'
import { Tree, TreeUpdate as TreeUpdateReq } from '@green-ecolution/backend-client'
import { useMutation, useQueryClient, useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useState } from 'react'

export const Route = createFileRoute('/_protected/map/sensor/select/tree/')({
  component: LinkTreeToSensor,
  beforeLoad: ({ search }) => {
    return search as {
      sensorId: string
    }
  },
  loaderDeps: ({ search: { sensorId } }) => ({ sensorId }),
  loader: ({ context: { queryClient }, deps: { sensorId } }) =>
    queryClient.prefetchQuery(sensorIdQuery(String(sensorId))),
})

function LinkTreeToSensor() {
  const queryClient = useQueryClient()
  const [treeId, setTreeId] = useState<number | undefined>()
  const [showDefault, setShowDefault] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const navigate = useNavigate({ from: Route.fullPath })
  const showToast = createToast()
  const { sensorId } = Route.useSearch()
  const { data: sensor } = useSuspenseQuery(sensorIdQuery(String(sensorId)))

  const { mutate } = useMutation({
    mutationFn: (tree: TreeUpdateReq) =>
      treeApi.updateTree({
        treeId: Number(treeId),
        body: tree,
      }),
    onSuccess: () => handleOnUpdateSuccess(),
    onError: (error: Error) => {
      console.error('Error updating tree data:', error)
      setErrorMessage('Es ist leider etwas schief gelaufen. Bitte probieren Sie es später erneut.')
    },
  })

  const handleOnUpdateSuccess = () => {
    handleNavigateBack()
      .then(() => showToast('Vegetation wurde erfolgreich verlinkt'))
      .catch(console.log)
  }

  const handleNavigateBack = useCallback(() => {
    return navigate({
      to: `/sensors/$sensorId`,
      params: { sensorId: sensorId?.toString() ?? '' },
    })
  }, [navigate, sensorId])

  const handleSave = async () => {
    setErrorMessage(null)

    if (!treeId) {
      setShowDefault(true)
      return
    }

    try {
      const data = await queryClient.fetchQuery(treeIdQuery(String(treeId)))
      mutate({
        ...data,
        sensorId: sensorId,
      })
    } catch (error) {
      console.error('Error fetching tree data:', error)
      setErrorMessage(
        'Die Baumdaten konnten nicht geladen werden. Bitte probieren Sie es später erneut.',
      )
    }
  }

  return (
    <>
      <MapSelectEntitiesModal
        onSave={() => void handleSave()}
        onCancel={() => void handleNavigateBack()}
        disabled={!treeId}
        title="Ausgewählte Bäume:"
        content={
          <>
            {!treeId || showDefault ? (
              <p className="text-dark-600 font-semibold text-sm">
                Hier können Sie die zugehörige Vegetation verlinken.
              </p>
            ) : (
              <SelectedCard id={treeId} type="tree" onClick={() => setTreeId(undefined)} />
            )}
            {errorMessage && <p className="text-red font-semibold text-sm mt-2">{errorMessage}</p>}
          </>
        }
      />
      <WithFilterableTrees
        selectedTrees={treeId ? [treeId] : []}
        onClick={(tree: Tree) => setTreeId(tree.id)}
      />
      <SensorMarker sensor={sensor} />
    </>
  )
}
