import { Ref, useEffect, useState } from 'react'
import FilterButton from '../buttons/FilterButton'
import PrimaryButton from '../buttons/PrimaryButton'
import SecondaryButton from '../buttons/SecondaryButton'
import { X } from 'lucide-react'
import { useNavigate } from '@tanstack/react-router'
import { useFilter, Filters } from '@/context/FilterContext'
import useStore from '@/store/store'

interface DialogProps {
  headline: string
  fullUrlPath: string
  children?: React.ReactNode
  isOnMap?: boolean
  onToggleOpen?: (isOpen: boolean) => void
  ref?: Ref<HTMLDivElement>
}

const Dialog = ({
  headline,
  fullUrlPath,
  children,
  isOnMap = false,
  ref,
  onToggleOpen,
}: DialogProps) => {
  const [isOpen, setIsOpen] = useState(false)
  const [oldValues, setOldValues] = useState<Filters>({
    statusTags: [],
    regionTags: [],
    hasCluster: undefined,
    plantingYears: [],
  })
  const [count, setCount] = useState(0)
  const navigate = useNavigate()
  const mapPosition = useStore((state) => ({
    lat: state.map.center[0],
    lng: state.map.center[1],
    zoom: state.map.zoom,
  }))

  const { filters, resetFilters, applyOldStateToTags } = useFilter()

  const handleSubmit = () => {
    setIsOpen(false)
    setIsOpen(false)
    navigate({
      to: fullUrlPath,
      search: () => ({
        lat: isOnMap ? mapPosition.lat : undefined,
        lng: isOnMap ? mapPosition.lng : undefined,
        zoom: isOnMap ? mapPosition.zoom : undefined,
        wateringStatuses: filters.statusTags.length > 0 ? filters.statusTags : undefined,
        regions: filters.regionTags.length > 0 ? filters.regionTags : undefined,
        hasCluster: filters.hasCluster ?? undefined,
        plantingYears: filters.plantingYears.length > 0 ? filters.plantingYears : undefined,
      }),
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const handleReset = () => {
    applyOldStateToTags({
      statusTags: [],
      regionTags: [],
      hasCluster: undefined,
      plantingYears: [],
    })
    resetFilters()
    setIsOpen(false)

    if (isOnMap) {
      navigate({
        to: fullUrlPath,
        search: {
          lat: mapPosition.lat,
          lng: mapPosition.lng,
          zoom: mapPosition.zoom,
        },
      }).catch((error) => console.error('Navigation failed:', error))
    } else {
      navigate({
        to: fullUrlPath,
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))
    }
  }

  const handleClose = () => {
    setIsOpen(false)
    applyOldStateToTags(oldValues)
  }

  const handleOpen = () => {
    setOldValues(filters)
    setIsOpen(true)
  }

  useEffect(() => {
    setCount(
      filters.statusTags.length +
        filters.regionTags.length +
        (filters.hasCluster !== undefined ? 1 : 0) +
        filters.plantingYears.length,
    )
  }, [filters])

  useEffect(() => {
    if (!onToggleOpen) return
    onToggleOpen(isOpen)
  }, [isOpen, onToggleOpen])

  return (
    <div className="font-nunito-sans text-base">
      <div
        onClick={() => handleClose()}
        className={`bg-dark-900/90 fixed inset-0 z-[1020] ${isOpen ? 'block' : 'hidden'}`}
      ></div>

      <FilterButton
        activeCount={count}
        ariaLabel={headline}
        isOnMap={isOnMap}
        onClick={() => {
          if (isOpen) {
            handleClose()
          } else {
            handleOpen()
          }
        }}
      />

      <section
        ref={ref}
        role="dialog"
        aria-modal="true"
        className={`fixed max-h-[80dvh] overflow-y-auto z-[1030] inset-x-4 shadow-xl bg-white top-1/2 -translate-y-1/2 p-6 rounded-xl mx-auto max-w-[30rem] ${isOpen ? 'block' : 'hidden'}`}
      >
        <div className="flex items-center justify-between gap-x-5 mb-5">
          <h2 className="text-xl font-lato font-semibold">{headline}</h2>
          <button
            aria-label="Close Dialog"
            type="reset"
            className="text-dark-400 hover:text-dark-600 stroke-1"
            onClick={handleClose}
          >
            <X />
          </button>
        </div>

        {children}

        <div className="flex flex-wrap gap-5 mt-6">
          <PrimaryButton label="Anwenden" type="button" onClick={handleSubmit} />
          <SecondaryButton label="Zurücksetzen" onClick={handleReset} />
        </div>
      </section>
    </div>
  )
}

export default Dialog
