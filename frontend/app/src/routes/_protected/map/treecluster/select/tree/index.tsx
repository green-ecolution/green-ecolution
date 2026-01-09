import { createFileRoute, useNavigate, useBlocker } from '@tanstack/react-router'
import { Tree } from '@green-ecolution/backend-client'
import { useCallback, useRef, useState } from 'react'
import SelectedCard from '@/components/general/cards/SelectedCard'
import WithAllTrees from '@/components/map/marker/WithAllTrees'
import MapSelectEntitiesModal from '@/components/map/MapSelectEntitiesModal'
import Modal from '@/components/general/Modal'
import { z } from 'zod'
import { safeJsonStorageParse } from '@/lib/utils'
import { clusterSchemaBase } from '@/schema/treeclusterSchema'

const selectTreesInClusterParams = z.object({
  formType: z.enum(['create', 'update']),
  treeIds: z.array(z.number().int()),
})

export const Route = createFileRoute('/_protected/map/treecluster/select/tree/')({
  validateSearch: selectTreesInClusterParams,
  loaderDeps: ({ search }) => search,
  component: SelectTrees,
})

function SelectTrees() {
  const { formType, treeIds: searchTreeIds } = Route.useSearch()
  const [treeIds, setTreeIds] = useState<number[]>(searchTreeIds)
  const [showError, setShowError] = useState(false)
  const navigate = useNavigate({ from: Route.fullPath })
  const { clusterId } = Route.useSearch()
  const allowNavigationRef = useRef(false)

  const { proceed, reset, status } = useBlocker({
    shouldBlockFn: ({ next }) => {
      if (allowNavigationRef.current) {
        allowNavigationRef.current = false
        return false
      }

      const isAllowedPath =
        next.pathname.startsWith('/treecluster/new') ||
        next.pathname.startsWith('/treecluster/') ||
        next.pathname.startsWith('/map/treecluster/select/tree')
      if (isAllowedPath) {
        return false
      }

      return true
    },
    withResolver: true,
  })

  const handleConfirmLeave = useCallback(() => {
    window.sessionStorage.removeItem('create-cluster')
    window.sessionStorage.removeItem('update-cluster')
    window.sessionStorage.removeItem('create-cluster-trees-changed')
    window.sessionStorage.removeItem('update-cluster-trees-changed')
    proceed?.()
  }, [proceed])

  const handleNavigateBack = useCallback(() => {
    allowNavigationRef.current = true
    switch (formType) {
      case 'create':
        return navigate({
          to: '/treecluster/new',
        })
      case 'update':
        return navigate({
          to: `/treecluster/$treeclusterId/edit`,
          params: { treeclusterId: clusterId?.toString() ?? '' },
        })
    }
  }, [navigate, formType, clusterId])

  const handleSave = () => {
    if (treeIds.length === 0) {
      setShowError(true)
      return
    }

    const { data, success, error } = safeJsonStorageParse(`${formType}-cluster`, {
      schema: clusterSchemaBase,
    })

    if (success) {
      const originalTreeIds = data.treeIds ?? []
      const treesChanged =
        treeIds.length !== originalTreeIds.length ||
        treeIds.some((id) => !originalTreeIds.includes(id))
      if (treesChanged) {
        window.sessionStorage.setItem(`${formType}-cluster-trees-changed`, 'true')
      }
      data.treeIds = treeIds
      window.sessionStorage.setItem(`${formType}-cluster`, JSON.stringify(data))
    } else {
      console.error(error)
    }

    handleNavigateBack().catch((error) => console.error('Navigation failed:', error))
  }

  const handleCancel = () => handleNavigateBack()

  const handleDeleteTree = (treeId: number) => {
    setTreeIds((prev) => prev.filter((id) => id !== treeId))
  }

  const handleTreeClick = (tree: Tree) => {
    if (treeIds.includes(tree.id)) setTreeIds((prev) => prev.filter((id) => id !== tree.id))
    else setTreeIds((prev) => [...prev, tree.id])
  }

  return (
    <>
      <MapSelectEntitiesModal
        onSave={handleSave}
        onCancel={() => void handleCancel()}
        disabled={treeIds.length === 0}
        title="Ausgewählte Bäume:"
        content={
          <ul>
            {(treeIds?.length || 0) === 0 || showError ? (
              <li className="text-dark-600 font-semibold text-sm">
                <p>Hier können Sie zugehörige Bäume verlinken.</p>
              </li>
            ) : (
              treeIds.map((treeId) => (
                <li key={treeId}>
                  <SelectedCard type="tree" id={treeId} onClick={handleDeleteTree} />
                </li>
              ))
            )}
          </ul>
        }
      />
      <WithAllTrees selectedTrees={treeIds} onClick={handleTreeClick} />

      <Modal
        title="Seite verlassen?"
        description="Möchtest du die Seite wirklich verlassen? Deine Eingaben gehen verloren, wenn du jetzt gehst."
        confirmText="Verlassen"
        isOpen={status === 'blocked'}
        onConfirm={handleConfirmLeave}
        onCancel={() => reset?.()}
      />
    </>
  )
}
