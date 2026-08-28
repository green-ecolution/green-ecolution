import { format } from 'date-fns'
import { StatusCard } from '@green-ecolution/ui'
import StatusCardGrid from '@/components/general/StatusCardGrid'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import type { Tree } from '@/api/backendApi'

interface TreeKpiRowProps {
  tree: Tree
}

const TreeKpiRow = ({ tree }: TreeKpiRowProps) => {
  const wateringStatus = getWateringStatusDetails(tree.wateringStatus)

  return (
    <StatusCardGrid className="lg:grid-cols-2">
      <li className="h-full">
        <StatusCard
          size="compact"
          status={wateringStatus.color}
          indicator="dot"
          label="Bewässerungszustand"
          value={wateringStatus.label}
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label="Letzte Bewässerung"
          value={tree.lastWatered ? format(new Date(tree.lastWatered), 'dd.MM.yyyy') : '—'}
          info="Wird aktualisiert, sobald ein Einsatzplan mit diesem Baum als »Beendet« markiert wird."
        />
      </li>
    </StatusCardGrid>
  )
}

export default TreeKpiRow
