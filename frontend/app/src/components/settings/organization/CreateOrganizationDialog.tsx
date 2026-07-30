import { useState, type FormEvent } from 'react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FormField,
} from '@green-ecolution/ui'

interface CreateOrganizationDialogProps {
  open: boolean
  parentName: string
  saving: boolean
  nameError: string | null
  onOpenChange: (open: boolean) => void
  onSubmit: (name: string) => void
}

const CreateOrganizationDialog = ({
  open,
  parentName,
  saving,
  nameError,
  onOpenChange,
  onSubmit,
}: CreateOrganizationDialogProps) => {
  const [name, setName] = useState('')

  const trimmed = name.trim()

  const handleOpenChange = (next: boolean) => {
    if (!next) setName('')
    onOpenChange(next)
  }

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault()
    // Disabling the button alone doesn't stop Enter-to-submit in every browser.
    if (trimmed === '' || saving) return
    onSubmit(trimmed)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Unterorganisation anlegen</DialogTitle>
          <DialogDescription>
            Die neue Organisation wird unter {parentName} eingeordnet.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <FormField
            id="new-organization-name"
            label="Name der Organisation"
            value={name}
            onChange={(e) => setName(e.target.value)}
            error={nameError ?? undefined}
            autoFocus
          />

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
              Abbrechen
            </Button>
            <Button type="submit" disabled={trimmed === '' || saving}>
              Anlegen
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default CreateOrganizationDialog
export type { CreateOrganizationDialogProps }
