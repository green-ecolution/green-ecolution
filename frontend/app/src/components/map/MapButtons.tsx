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

const MapButtons = () => {
  const [isModalOpen, setIsModalOpen] = useState(false)
  return (
    <>
      <Button
        variant="outline"
        size="icon"
        aria-label="Kataster-Einstellungen"
        onClick={() => setIsModalOpen(!isModalOpen)}
        className="rounded-full shadow-cards bg-white border-dark-200"
      >
        <Settings className="!size-5 text-dark-800" />
      </Button>
      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Weitere Kataster-Einstellungen</DialogTitle>
            <DialogDescription>
              In dieser Ansicht können weitere Einstellungen vorgenommen werden. Es können zum
              Beispiel manuell Bäume zum Kataster hinzugefügt werden.
            </DialogDescription>
          </DialogHeader>
          <Link
            to="/map/tree/new"
            preload="intent"
            search={(prev) => prev}
            onClick={() => setIsModalOpen(false)}
            className="group flex items-center gap-x-2 !text-green-dark font-medium text-base mb-4"
          >
            Neuen Baum manuell hinzufügen
            <MoveRight className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:translate-x-1" />
          </Link>
        </DialogContent>
      </Dialog>
    </>
  )
}

export default MapButtons
