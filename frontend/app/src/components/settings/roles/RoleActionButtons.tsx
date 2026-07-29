import { Button, Spinner } from '@green-ecolution/ui'

interface RoleActionButtonsProps {
  isNew: boolean
  saving: boolean
  nameEmpty: boolean
  onSave: () => void
  onCancel: () => void
}

const RoleActionButtons = ({ isNew, saving, nameEmpty, onSave, onCancel }: RoleActionButtonsProps) => (
  <>
    <Button
      type="button"
      variant="outline"
      onClick={onCancel}
      disabled={saving}
      className="w-full sm:w-auto"
    >
      Abbrechen
    </Button>
    <Button
      type="button"
      onClick={onSave}
      disabled={saving || nameEmpty}
      className="w-full sm:w-auto"
    >
      {saving && <Spinner className="size-4" />}
      {isNew ? 'Rolle anlegen' : 'Speichern'}
    </Button>
  </>
)

export default RoleActionButtons
