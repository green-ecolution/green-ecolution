import { MoveRight, Settings } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@green-ecolution/ui'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Can } from '@/lib/auth/Can'

const MapButtons = () => {
  const { t } = useTranslation('map')
  const [isModalOpen, setIsModalOpen] = useState(false)
  return (
    <Can permission={['tree:create']}>
      <Button
        variant="outline"
        size="icon"
        aria-label={t('toolbar.katasterSettingsAriaLabel')}
        onClick={() => setIsModalOpen(!isModalOpen)}
        className="rounded-full shadow-cards bg-white border-dark-200"
      >
        <Settings className="!size-5 text-dark-800" />
      </Button>
      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('toolbar.katasterSettingsTitle')}</DialogTitle>
            <DialogDescription>{t('toolbar.katasterSettingsDescription')}</DialogDescription>
          </DialogHeader>
          <Link
            to="/map/tree/new"
            preload="intent"
            search={(prev) => prev}
            onClick={() => setIsModalOpen(false)}
            className="group flex items-center gap-x-2 !text-green-dark font-medium text-base mb-4"
          >
            {t('toolbar.addTreeManually')}
            <MoveRight className="w-4 h-4 transition-transform duration-base ease-emphasized group-hover:translate-x-1 motion-reduce:transition-none" />
          </Link>
        </DialogContent>
      </Dialog>
    </Can>
  )
}

export default MapButtons
