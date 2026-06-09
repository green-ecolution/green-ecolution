import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Loading } from '@green-ecolution/ui'
import { treeClusterIdQuery } from '@/api/queries'
import ClusterPanelShell from './ClusterPanelShell'
import ClusterPanelView from './ClusterPanelView'
import ClusterPanelEdit from './ClusterPanelEdit'

interface ClusterPanelProps {
  clusterId: string
  onClose: () => void
  onOpenDashboard: () => void
}

const ClusterPanel = ({ clusterId, onClose, onOpenDashboard }: ClusterPanelProps) => {
  const [mode, setMode] = useState<'view' | 'edit'>('view')
  const { data, isPending, isError } = useQuery(treeClusterIdQuery(clusterId))

  return (
    <ClusterPanelShell onClose={onClose}>
      {isPending && <Loading className="justify-center py-10" label="Lade Baumgruppe..." />}
      {isError && (
        <p className="py-10 text-center text-dark-600">
          Die Baumgruppe konnte nicht geladen werden.
        </p>
      )}
      {data &&
        (mode === 'view' ? (
          <ClusterPanelView
            treecluster={data}
            onEdit={() => setMode('edit')}
            onOpenDashboard={onOpenDashboard}
          />
        ) : (
          <ClusterPanelEdit
            treecluster={data}
            onCancel={() => setMode('view')}
            onSaved={() => setMode('view')}
          />
        ))}
    </ClusterPanelShell>
  )
}

export default ClusterPanel
