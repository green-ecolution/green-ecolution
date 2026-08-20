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

const describe = ({
  organizationName,
  canSwitch,
  blockedReason,
  selectedTreeCount,
}: Pick<
  ForeignTreeDialogProps,
  'organizationName' | 'canSwitch' | 'blockedReason' | 'selectedTreeCount'
>) => {
  const owner = organizationName
    ? `Dieser Baum gehört der Organisation ${organizationName}.`
    : 'Dieser Baum gehört einer anderen Organisation.'

  if (!canSwitch) {
    return `${owner} Eine Bewässerungsgruppe darf nur Bäume ihrer eigenen Organisation enthalten. ${blockedReason}`
  }

  const consequence =
    selectedTreeCount > 0
      ? ` Die bisher ausgewählten Bäume (${selectedTreeCount}) werden dabei verworfen, weil eine Gruppe nur Bäume einer Organisation enthalten darf.`
      : ''

  return `${owner} Soll die Gruppe für diese Organisation angelegt werden?${consequence}`
}

const ForeignTreeDialog = (props: ForeignTreeDialogProps) => (
  <AlertDialog open={props.open} onOpenChange={props.onOpenChange}>
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>
          {props.canSwitch ? 'Organisation wechseln?' : 'Baum nicht auswählbar'}
        </AlertDialogTitle>
        <AlertDialogDescription>{describe(props)}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>
          {props.canSwitch ? 'Abbrechen' : 'Verstanden'}
          <X />
        </AlertDialogCancel>
        {props.canSwitch && (
          <AlertDialogAction onClick={props.onConfirm}>
            Wechseln
            <MoveRight className="icon-arrow-animate" />
          </AlertDialogAction>
        )}
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
)

export default ForeignTreeDialog
