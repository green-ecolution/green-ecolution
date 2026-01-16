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
    <div className="relative">
      <div className="absolute z-[1000] space-y-2 top-6 left-36 flex space-x-2">
        <Button
          variant="outline"
          size="icon"
          onClick={() => setIsModalOpen(!isModalOpen)}
          className="rounded-full shadow-cards bg-white"
        >
          <Settings className="!size-6 text-dark-800" />
        </Button>
      </div>
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
            className="group flex items-center gap-x-2 !text-green-dark font-medium text-base mb-4"
          >
            Neuen Baum manuell hinzufügen
            <MoveRight className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:translate-x-1" />
          </Link>
        </DialogContent>
      </Dialog>
    </div>
  )
}

export default MapButtons
