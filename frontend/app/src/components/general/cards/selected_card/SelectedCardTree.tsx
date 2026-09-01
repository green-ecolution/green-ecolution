import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { SelectedCardProps } from '../SelectedCard'
import { treeQueries } from '@/api/queries'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
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
  const { t } = useTranslation(['tree', 'common'])
  // useQuery (not suspense) so adding a card doesn't suspend and flicker the panel.
  const { data } = useQuery(treeQueries.detail(String(id)))
  const getWateringStatusDetails = useWateringStatusDetails()
  const statusDetails = getWateringStatusDetails(data?.wateringStatus ?? 'unknown')

  return (
    <ListCard size="compact" hoverable={false} className="mb-3">
      <ListCardStatus status={statusDetails.color} />
      <ListCardContent>
        <span className="font-medium">
          <strong className="font-semibold">{t('selectedCard.label')}</strong>
          {data ? (
            <>
              &nbsp;{data.species} · {data.number} · {data.plantingYear}
            </>
          ) : (
            <>&nbsp;{t('common:state.loadingInline')}</>
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
            <span className="sr-only">{t('selectedCard.removeAriaLabel')}</span>
          </Button>
        </ListCardActions>
      )}
    </ListCard>
  )
}

export default SelectedCardTree
