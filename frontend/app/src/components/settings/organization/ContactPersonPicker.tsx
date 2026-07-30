import {
  Avatar,
  AvatarFallback,
  Button,
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@green-ecolution/ui'
import type { UserResponse } from '@/api/backendApi'

interface ContactPersonPickerProps {
  open: boolean
  members: UserResponse[]
  selectedId: string | null
  onOpenChange: (open: boolean) => void
  onSelect: (userId: string | null) => void
}

const EMPTY_TEXT = 'Es sind keine Personen dieser Organisation zugeordnet.'

const ContactPersonPicker = ({
  open,
  members,
  selectedId,
  onOpenChange,
  onSelect,
}: ContactPersonPickerProps) => (
  <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Kontaktperson wählen</DialogTitle>
        <DialogDescription>
          Nur Personen, die dieser Organisation zugeordnet sind, können Kontaktperson sein.
        </DialogDescription>
      </DialogHeader>

      <Command>
        <CommandInput placeholder="Person suchen" />
        <CommandList>
          <CommandEmpty>{EMPTY_TEXT}</CommandEmpty>
          {members.map((member) => (
            <CommandItem
              key={member.id}
              value={`${member.firstName} ${member.lastName} ${member.email}`}
              onSelect={() => {
                onSelect(member.id)
                onOpenChange(false)
              }}
            >
              <Avatar size="sm">
                <AvatarFallback variant="user">
                  {`${member.firstName.charAt(0)}${member.lastName.charAt(0)}`.toUpperCase()}
                </AvatarFallback>
              </Avatar>
              <span className="flex min-w-0 flex-col">
                <span className="truncate">{`${member.firstName} ${member.lastName}`}</span>
                <span className="truncate text-xs text-dark-400">{member.email}</span>
              </span>
            </CommandItem>
          ))}
        </CommandList>
      </Command>

      {selectedId !== null && (
        <DialogFooter>
          <Button variant="outline" onClick={() => onSelect(null)}>
            Kontaktperson entfernen
          </Button>
        </DialogFooter>
      )}
    </DialogContent>
  </Dialog>
)

export default ContactPersonPicker
export type { ContactPersonPickerProps }
