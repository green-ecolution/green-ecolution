import { MoveRight, Settings } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import Modal from '../general/Modal'
import { useState } from 'react'

const MapButtons = () => {
  const [isModalOpen, setIsModalOpen] = useState(false)
  return (
    <div className="relative">
      <div className="absolute z-[1000] space-y-2 top-6 left-36 flex space-x-2">
        <button
          type="button"
          onClick={() => setIsModalOpen(!isModalOpen)}
          className={`bg-white shadow-cards w-10 h-10 rounded-full flex items-center justify-center transition-all ease-in-out duration-300`}
        >
          <Settings className="w-6 h-6 text-dark-800" />
        </button>
      </div>
      <Modal
        title="Weitere Kataster-Einstellungen"
        description="In dieser Ansicht können weitere Einstellungen vorgenommen werden. Es können zum Beispiel manuell Bäume zum Kataster hinzugefügt werden."
        confirmText="Import fortfahren"
        showButtons={false}
        onCancel={() => setIsModalOpen(false)}
        isOpen={isModalOpen}
      >
        <Link
          to="/map/tree/new"
          preload="intent"
          search={(prev) => prev}
          className="group flex items-center gap-x-2 !text-green-dark font-medium text-base mb-4"
        >
          Neuen Baum manuell hinzufügen
          <MoveRight className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:translate-x-1" />
        </Link>
      </Modal>
    </div>
  )
}

export default MapButtons
