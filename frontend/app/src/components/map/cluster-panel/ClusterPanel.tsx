import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Loading } from '@green-ecolution/ui'
import { isValidUuid, treeClusterIdQuery } from '@/api/queries'
import ClusterPanelShell from './ClusterPanelShell'
import ClusterPanelView from './ClusterPanelView'
import ClusterPanelEdit from './ClusterPanelEdit'

interface ClusterPanelProps {
  clusterId: string
  onClose: () => void
  onOpenDashboard: () => void
  onExpand?: () => void
}

const ClusterPanel = ({ clusterId, onClose, onOpenDashboard, onExpand }: ClusterPanelProps) => {
  const [mode, setMode] = useState<'view' | 'edit'>('view')
  const { data, isError } = useQuery(treeClusterIdQuery(clusterId))
  const failed = !isValidUuid(clusterId) || isError

  return (
    <ClusterPanelShell onClose={onClose}>
      {data ? (
        mode === 'view' ? (
          <ClusterPanelView
            treecluster={data}
            onEdit={() => {
              setMode('edit')
              onExpand?.()
            }}
            onClose={onClose}
            onOpenDashboard={onOpenDashboard}
          />
        ) : (
          <ClusterPanelEdit
            treecluster={data}
            onCancel={() => setMode('view')}
            onClose={onClose}
            onSaved={() => setMode('view')}
          />
        )
      ) : failed ? (
        <p className="py-10 text-center text-dark-600">
          Die Baumgruppe konnte nicht geladen werden.
        </p>
      ) : (
        <Loading className="justify-center py-10" label="Lade Baumgruppe..." />
      )}
    </ClusterPanelShell>
  )
}

export default ClusterPanel
