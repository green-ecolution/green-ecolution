import { Ref, useEffect, useMemo, useState } from 'react'
import FilterButton from '../buttons/FilterButton'
import PrimaryButton from '../buttons/PrimaryButton'
import SecondaryButton from '../buttons/SecondaryButton'
import { useNavigate } from '@tanstack/react-router'
import { useFilter, Filters } from '@/context/FilterContext'
import useStore from '@/store/store'
import { BaseModal } from '../modal'

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
  onToggleOpen,
}: DialogProps) => {
  const [isOpen, setIsOpen] = useState(false)
  const [oldValues, setOldValues] = useState<Filters>({
    statusTags: [],
    regionTags: [],
    hasCluster: undefined,
    plantingYears: [],
  })
  const navigate = useNavigate()
  const mapCenter = useStore((state) => state.mapCenter)
  const mapZoom = useStore((state) => state.mapZoom)
  const mapPosition = { lat: mapCenter[0], lng: mapCenter[1], zoom: mapZoom }

  const { filters, resetFilters, applyOldStateToTags } = useFilter()

  const handleSubmit = () => {
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

  const count = useMemo(
    () =>
      filters.statusTags.length +
      filters.regionTags.length +
      (filters.hasCluster !== undefined ? 1 : 0) +
      filters.plantingYears.length,
    [filters],
  )

  useEffect(() => {
    if (!onToggleOpen) return
    onToggleOpen(isOpen)
  }, [isOpen, onToggleOpen])

  return (
    <div className="font-nunito-sans text-base">
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

      <BaseModal
        isOpen={isOpen}
        onClose={handleClose}
        title={headline}
        className="max-h-[80dvh] overflow-y-auto"
      >
        {children}

        <div className="flex flex-wrap gap-5 mt-6">
          <PrimaryButton label="Anwenden" type="button" onClick={handleSubmit} />
          <SecondaryButton label="Zurücksetzen" onClick={handleReset} />
        </div>
      </BaseModal>
    </div>
  )
}

export default Dialog
