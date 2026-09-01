import { clusterQueries } from '@/api/queries'
import { SelectedCardProps } from '../SelectedCard'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation(['treecluster', 'common'])
  const { data } = useQuery(clusterQueries.detail(String(id)))
  const statusDetails = getWateringStatusDetails(data?.wateringStatus ?? 'unknown')

  return (
    <ListCard size="compact" hoverable={false} className="mb-3">
      <ListCardStatus status={statusDetails.color} />
      <ListCardContent>
        {data ? (
          <div className="min-w-0">
            <p className="truncate font-semibold text-dark">{data.name}</p>
            <p className="truncate text-xs tabular-nums text-dark-600">
              {t('selectedCard.treeCount', { count: data.trees.length })}
              {data.address && <> · {data.address}</>}
            </p>
          </div>
        ) : (
          <span className="text-dark-600">{t('common:state.loadingInline')}</span>
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
            <span className="sr-only">{t('selectedCard.removeAriaLabel')}</span>
          </Button>
        </ListCardActions>
      )}
    </ListCard>
  )
}

export default SelectedCardCluster
