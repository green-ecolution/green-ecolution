import { treeClusterIdQuery } from '@/api/queries'
import { SelectedCardProps } from '../SelectedCard'
import { useQuery } from '@tanstack/react-query'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { Trash2 } from 'lucide-react'
import {
  ListCard,
  ListCardStatus,
  ListCardContent,
  ListCardActions,
  Button,
} from '@green-ecolution/ui'

interface SelectedCardClusterProps extends Omit<SelectedCardProps, 'type'> {}

const SelectedCardCluster = ({ onClick, id }: SelectedCardClusterProps) => {
  const { data } = useQuery(treeClusterIdQuery(String(id)))
  const statusDetails = getWateringStatusDetails(data?.wateringStatus ?? 'unknown')

  return (
    <ListCard size="compact" hoverable={false} className="mb-3">
      <ListCardStatus status={statusDetails.color} />
      <ListCardContent>
        {data ? (
          <div className="min-w-0">
            <p className="truncate font-semibold text-dark">{data.name}</p>
            <p className="truncate text-xs tabular-nums text-dark-600">
              {data.trees.length} {data.trees.length === 1 ? 'Baum' : 'Bäume'}
              {data.address && <> · {data.address}</>}
            </p>
          </div>
        ) : (
          <span className="text-dark-600">Lädt…</span>
        )}
      </ListCardContent>
      {onClick && (
        <ListCardActions>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="text-dark-600 hover:text-destructive"
            onClick={() => onClick(id)}
          >
            <Trash2 className="w-5 h-5" />
            <span className="sr-only">Bewässerungsgruppe aus Auswahl löschen</span>
          </Button>
        </ListCardActions>
      )}
    </ListCard>
  )
}

export default SelectedCardCluster
