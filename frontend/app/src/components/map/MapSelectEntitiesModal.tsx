import React, { useEffect, useState, Ref, useCallback, useId } from 'react'
import { Button } from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'
import useMapInteractions from '@/hooks/useMapInteractions'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useFocusTrap } from '@/hooks/useFocusTrap'

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
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const titleId = useId()

  const focusTrapRef = useFocusTrap<HTMLDivElement>({
    enabled: openModal && !isLargeScreen,
    returnFocusOnDeactivate: true,
  })

  const handleEscapeKey = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape' && openModal && !isLargeScreen) {
        setOpenModal(false)
      }
    },
    [openModal, isLargeScreen],
  )

  useEffect(() => {
    if (isLargeScreen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- sync state with media query
      setOpenModal(false)
      enableDragging()
    }
  }, [isLargeScreen, enableDragging])

  useEffect(() => {
    if (openModal && !isLargeScreen) {
      disableDragging()
    } else if (!isLargeScreen) {
      enableDragging()
    }
  }, [disableDragging, enableDragging, openModal, isLargeScreen])

  useEffect(() => {
    if (openModal && !isLargeScreen) {
      document.addEventListener('keydown', handleEscapeKey)
      return () => document.removeEventListener('keydown', handleEscapeKey)
    }
  }, [openModal, isLargeScreen, handleEscapeKey])

  const modalContent = (
    <>
      <div className="max-h-[40vh] overflow-y-auto">{content}</div>
      <div className="flex flex-wrap gap-5 mt-6">
        <Button type="submit" onClick={onSave} disabled={disabled}>
          Speichern
          <MoveRight />
        </Button>
        <Button variant="outline" onClick={onCancel}>
          Zurück
          <X />
        </Button>
      </div>
    </>
  )

  const mobileModal = (
    <>
      <div
        onClick={() => setOpenModal(false)}
        className={`bg-dark-900/90 fixed inset-0 z-[1020] lg:hidden ${openModal ? 'block' : 'hidden'}`}
        aria-hidden="true"
      />
      <div
        ref={focusTrapRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={`space-y-6 fixed z-[1030] top-20 inset-x-4 bg-white border-dark-50 shadow-xl p-5 rounded-xl font-nunito-sans lg:hidden ${openModal ? 'block' : 'hidden'}`}
      >
        <div className="flex justify-between gap-x-6">
          <h2 id={titleId} className="text-lg font-lato font-semibold">
            {title}
          </h2>
          <button
            aria-label="Dialog schließen"
            className="text-dark-400 hover:text-dark-600 stroke-1"
            type="button"
            onClick={() => setOpenModal(false)}
          >
            <X />
          </button>
        </div>
        {modalContent}
      </div>
    </>
  )

  return (
    <div ref={ref} className="text-base font-nunito-sans">
      {/* Mobile: Button to open modal */}
      <button
        type="button"
        onClick={() => setOpenModal(true)}
        className={`bg-white absolute flex items-center group gap-x-3 rounded-xl px-5 py-2 z-[1000] left-4 bottom-6 transition-all ease-in-out duration-300 hover:bg-dark-100 lg:hidden
          ${openModal ? 'hidden' : ''}`}
      >
        <span className="font-medium text-base">Auswahl anzeigen</span>
        <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
      </button>

      {/* Mobile: Modal */}
      {mobileModal}

      {/* Desktop: Always visible sidebar */}
      {isLargeScreen && (
        <div
          role="dialog"
          onMouseEnter={disableDragging}
          onMouseLeave={enableDragging}
          aria-modal="false"
          className="space-y-6 absolute z-[1030] top-4 w-[35rem] right-10 bg-white border-dark-50 shadow-xl p-5 rounded-xl"
        >
          <div className="flex justify-between gap-x-6">
            <h2 className="text-lg font-lato font-semibold lg:text-xl">{title}</h2>
          </div>
          {modalContent}
        </div>
      )}
    </div>
  )
}

export default MapSelectEntitiesModal
