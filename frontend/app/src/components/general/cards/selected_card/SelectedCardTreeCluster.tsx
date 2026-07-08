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
        <span className="font-medium">
          <strong className="font-semibold">Bewässerungsgruppe:</strong>
          {data ? (
            <>
              &nbsp;{data.name} · {id}
            </>
          ) : (
            <>&nbsp;Lädt…</>
          )}
        </span>
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
