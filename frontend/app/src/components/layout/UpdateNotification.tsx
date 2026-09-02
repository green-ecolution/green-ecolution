import { RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('navigation')

  return (
    <Dialog open={updateAvailable} onOpenChange={(open) => !open && dismissUpdate()}>
      <DialogContent>
        <DialogIcon variant="info">
          <RefreshCw />
        </DialogIcon>
        <DialogHeader>
          <DialogTitle>{t('updateNotification.title')}</DialogTitle>
          <DialogDescription>{t('updateNotification.description')}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={dismissUpdate}>
            {t('updateNotification.later')}
          </Button>
          <Button onClick={performUpdate}>{t('updateNotification.updateNow')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
