import type { TreeCluster } from '@/api/backendApi'
import { clusterSignalSummary } from './clusterSignalSummary'

interface ClusterSignalCardProps {
  treecluster: TreeCluster
}

const ClusterSignalCard = ({ treecluster }: ClusterSignalCardProps) => {
  const s = clusterSignalSummary(treecluster.trees)

  if (s.total === 0) {
    return (
      <div className="h-full flex flex-col gap-y-3 rounded-xl bg-dark-50 p-6">
        <p className="text-sm text-dark-700 font-medium">Signal in der Gruppe</p>
        <p className="text-dark-600">Keine Sensoren mit Signaldaten.</p>
      </div>
    )
  }

  const pct = (n: number) => `${(n / s.total) * 100}%`

  return (
    <div className="h-full flex flex-col gap-y-3 rounded-xl bg-dark-50 p-6">
      <p className="text-sm text-dark-700 font-medium">Signal in der Gruppe</p>
      <p className="text-3xl font-bold">
        {s.total} Sensor{s.total === 1 ? '' : 'en'}
      </p>
      <span className="flex h-2.5 overflow-hidden rounded-full bg-dark-200">
        <span className="bg-green-dark" style={{ width: pct(s.good) }} />
        <span className="bg-yellow" style={{ width: pct(s.fair) }} />
        <span className="bg-red" style={{ width: pct(s.weak) }} />
      </span>
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-dark-700">
        <span>
          <span className="mr-1 inline-block size-2 rounded-sm bg-green-dark align-middle" />
          {s.good} gut
        </span>
        <span>
          <span className="mr-1 inline-block size-2 rounded-sm bg-yellow align-middle" />
          {s.fair} ausreichend
        </span>
        <span>
          <span className="mr-1 inline-block size-2 rounded-sm bg-red align-middle" />
          {s.weak} schwach
        </span>
      </div>
      {s.weakest && (
        <p className="text-xs text-dark-600">
          Schwächster: Baum {s.weakest.treeNumber} · {s.weakest.rssiDbm} dBm
        </p>
      )}
    </div>
  )
}

export default ClusterSignalCard
