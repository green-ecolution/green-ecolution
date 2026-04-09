import { RefreshCw } from 'lucide-react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogIcon,
  DialogTitle,
} from '@green-ecolution/ui'
import useAppUpdate from '@/hooks/useAppUpdate'

export default function UpdateNotification() {
  const { updateAvailable, performUpdate, dismissUpdate } = useAppUpdate()

  return (
    <Dialog open={updateAvailable} onOpenChange={(open) => !open && dismissUpdate()}>
      <DialogContent>
        <DialogIcon variant="info">
          <RefreshCw />
        </DialogIcon>
        <DialogHeader>
          <DialogTitle>Neue Version verfügbar</DialogTitle>
          <DialogDescription>
            Eine neue Version von Green Ecolution ist verfügbar. Aktualisiere die Anwendung, um die
            neuesten Verbesserungen zu nutzen.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={dismissUpdate}>
            Später
          </Button>
          <Button onClick={performUpdate}>Jetzt aktualisieren</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
