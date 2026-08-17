import { Button, Spinner } from '@green-ecolution/ui'

interface OrganizationActionButtonsProps {
  saving: boolean
  nameEmpty: boolean
  addressComplete: boolean
  onSave: () => void
  onCancel: () => void
}

const OrganizationActionButtons = ({
  saving,
  nameEmpty,
  addressComplete,
  onSave,
  onCancel,
}: OrganizationActionButtonsProps) => (
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
      disabled={saving || nameEmpty || !addressComplete}
      className="w-full sm:w-auto"
    >
      {saving && <Spinner className="size-4" />}
      Speichern
    </Button>
  </>
)

export default OrganizationActionButtons
