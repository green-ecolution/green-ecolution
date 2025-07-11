import { treeClusterIdQuery } from '@/api/queries'
import { SelectedCardProps } from '../SelectedCard'
import { useSuspenseQuery } from '@tanstack/react-query'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { Trash2 } from 'lucide-react'

interface SelectedCardClusterProps extends Omit<SelectedCardProps, 'type'> {}

const SelectedCardCluster = ({ onClick, id }: SelectedCardClusterProps) => {
  const { data } = useSuspenseQuery(treeClusterIdQuery(String(id)))

  const statusDetails = getWateringStatusDetails(data.wateringStatus)
  return (
    <div className="flex h-max justify-between mb-3 gap-x-6 bg-white border border-dark-50 shadow-cards px-4 py-3 rounded-lg">
      <h3
        className={`relative flex flex-wrap font-medium pl-7 before:absolute before:w-4 before:h-4 before:rounded-full before:left-0 before:top-[0.22rem] before:bg-${statusDetails.color}`}
      >
        <strong className="font-semibold">Bewässerungsgruppe:</strong>
        &nbsp;
        {data.name} · {id}
      </h3>
      {onClick && (
        <button
          type="button"
          onClick={() => {
            onClick(id)
          }}
          className="text-dark-600"
        >
          <Trash2 className="w-5 h-5" />
          <span className="sr-only">{'Bewässerungsgruppe'}&nbsp; aus Auswahl löschen</span>
        </button>
      )}
    </div>
  )
}

export default SelectedCardCluster
