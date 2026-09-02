import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'

interface ForeignTreeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Name of the tree's organization; undefined when it is not resolvable. */
  organizationName?: string
  /** Whether the group may move to that organization. */
  canSwitch: boolean
  /** Shown instead of the switch offer when `canSwitch` is false. */
  blockedReason: string
  /** Trees that switching would drop from the current selection. */
  selectedTreeCount: number
  onConfirm: () => void
}

const describe = (
  t: TFunction<['tree', 'common']>,
  {
    organizationName,
    canSwitch,
    blockedReason,
    selectedTreeCount,
  }: Pick<
    ForeignTreeDialogProps,
    'organizationName' | 'canSwitch' | 'blockedReason' | 'selectedTreeCount'
  >,
) => {
  const owner = organizationName
    ? t('foreignDialog.ownedBy', { organization: organizationName })
    : t('foreignDialog.ownedByOther')

  if (!canSwitch) {
    return `${owner} ${t('foreignDialog.blockedExplanation')} ${blockedReason}`
  }

  const consequence =
    selectedTreeCount > 0 ? ` ${t('foreignDialog.consequence', { count: selectedTreeCount })}` : ''

  return `${owner} ${t('foreignDialog.confirmQuestion')}${consequence}`
}

const ForeignTreeDialog = (props: ForeignTreeDialogProps) => {
  const { t } = useTranslation(['tree', 'common'])

  return (
    <AlertDialog open={props.open} onOpenChange={props.onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {props.canSwitch ? t('foreignDialog.titleSwitch') : t('foreignDialog.titleBlocked')}
          </AlertDialogTitle>
          <AlertDialogDescription>{describe(t, props)}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>
            {props.canSwitch ? t('common:actions.cancel') : t('foreignDialog.understood')}
            <X />
          </AlertDialogCancel>
          {props.canSwitch && (
            <AlertDialogAction onClick={props.onConfirm}>
              {t('foreignDialog.switchAction')}
              <MoveRight className="icon-arrow-animate" />
            </AlertDialogAction>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

export default ForeignTreeDialog
