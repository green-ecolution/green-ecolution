import { useQuery } from '@tanstack/react-query'
import { SelectedCardProps } from '../SelectedCard'
import { treeIdQuery } from '@/api/queries'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { Trash2 } from 'lucide-react'
import {
  ListCard,
  ListCardStatus,
  ListCardContent,
  ListCardActions,
  Button,
} from '@green-ecolution/ui'

interface SelectedCardTreeProps extends Omit<SelectedCardProps, 'type'> {}

const SelectedCardTree = ({ onClick, id }: SelectedCardTreeProps) => {
  // useQuery (not suspense) so adding a card doesn't suspend and flicker the panel.
  const { data } = useQuery(treeIdQuery(String(id)))
  const statusDetails = getWateringStatusDetails(data?.wateringStatus ?? 'unknown')

  return (
    <ListCard size="compact" hoverable={false} className="mb-3">
      <ListCardStatus status={statusDetails.color} />
      <ListCardContent>
        <span className="font-medium">
          <strong className="font-semibold">Baum:</strong>
          {data ? (
            <>
              &nbsp;{data.species} · {data.number} · {data.plantingYear}
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
            <span className="sr-only">Baum aus Auswahl löschen</span>
          </Button>
        </ListCardActions>
      )}
    </ListCard>
  )
}

export default SelectedCardTree
