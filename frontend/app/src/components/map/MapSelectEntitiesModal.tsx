import React, { useEffect, useState, Ref } from 'react'
import PrimaryButton from '../general/buttons/PrimaryButton'
import SecondaryButton from '../general/buttons/SecondaryButton'
import { MoveRight, X } from 'lucide-react'
import useMapInteractions from '@/hooks/useMapInteractions'

interface MapSelectEntitiesModalProps {
  onSave: () => void
  onCancel: () => void
  title?: string
  subtitle?: string
  content: React.ReactNode
  disabled?: boolean
  ref?: Ref<HTMLDivElement>
}

const MapSelectEntitiesModal = ({
  onSave,
  onCancel,
  content,
  title,
  ref,
  disabled = false,
}: MapSelectEntitiesModalProps) => {
  const [openModal, setOpenModal] = useState(false)
  const { enableDragging, disableDragging } = useMapInteractions()

  useEffect(() => {
    const handleResize = () => {
      if (window.matchMedia('(min-width: 1024px)').matches) {
        setOpenModal(false)
        enableDragging()
      }
    }

    window.addEventListener('resize', handleResize)
    handleResize()

    return () => {
      window.removeEventListener('resize', handleResize)
    }
  }, [enableDragging])

  useEffect(() => {
    if (openModal) {
      disableDragging()
    } else {
      enableDragging()
    }
  }, [disableDragging, enableDragging, openModal])

  return (
    <div ref={ref} className="text-base font-nunito-sans">
      <div
        onClick={() => setOpenModal(false)}
        className={`bg-dark-900/90 fixed inset-0 z-[1020] lg:hidden ${openModal ? 'block' : 'hidden'}`}
      ></div>
      <button
        type="button"
        onClick={() => setOpenModal(true)}
        className={`bg-white absolute flex items-center group gap-x-3 rounded-xl px-5 py-2 z-[1000] left-4 bottom-6 transition-all ease-in-out duration-300 hover:bg-dark-100 lg:hidden
          ${openModal ? 'hidden' : ''}`}
      >
        <span className="font-medium text-base">Auswahl anzeigen</span>
        <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
      </button>

      <div
        role="dialog"
        onMouseEnter={disableDragging}
        onMouseLeave={enableDragging}
        aria-modal="true"
        className={`space-y-6 absolute z-[1030] top-4 inset-x-4 lg:w-[35rem] lg:left-auto lg:right-10 bg-white border-dark-50 shadow-xl p-5 rounded-xl
          ${openModal ? 'block' : 'hidden lg:block'}`}
      >
        <div className="flex justify-between gap-x-6">
          <h2 className="text-lg font-lato font-semibold lg:text-xl">{title}</h2>
          <button
            aria-label="Dialog schließen"
            className="text-dark-400 hover:text-dark-600 stroke-1 lg:hidden"
            type="button"
            onClick={() => setOpenModal(false)}
          >
            <X />
          </button>
        </div>

        <div className="max-h-[40vh] overflow-y-auto">{content}</div>

        <div className="flex flex-wrap gap-5">
          <PrimaryButton type="submit" label="Speichern" onClick={onSave} disabled={disabled} />
          <SecondaryButton label="Zurück" onClick={onCancel} />
        </div>
      </div>
    </div>
  )
}

export default MapSelectEntitiesModal
