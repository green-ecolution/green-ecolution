import { useState } from 'react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
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

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Unterorganisation anlegen</DialogTitle>
          <DialogDescription>
            Die neue Organisation wird unter {parentName} eingeordnet.
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-2">
          <Label htmlFor="new-organization-name">Name der Organisation</Label>
          <Input
            id="new-organization-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />
          {nameError && <p className="text-sm text-destructive">{nameError}</p>}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
            Abbrechen
          </Button>
          <Button
            type="button"
            disabled={trimmed === '' || saving}
            onClick={() => onSubmit(trimmed)}
          >
            Anlegen
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default CreateOrganizationDialog
export type { CreateOrganizationDialogProps }
