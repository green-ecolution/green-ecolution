import { Ref, useEffect, useMemo, useState } from 'react'
import FilterButton from '../buttons/FilterButton'
import {
  Button,
  Dialog as DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { useFilter } from '@/context/FilterContext'
import { filtersFromSearch, searchFromFilters } from '@/lib/filterSearchSchema'

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
  const navigate = useNavigate()
  const search = useSearch({ strict: false })
  const { filters, resetFilters, applyOldStateToTags } = useFilter()

  const appliedFilters = useMemo(() => filtersFromSearch(search), [search])

  const handleSubmit = () => {
    setIsOpen(false)
    navigate({
      to: fullUrlPath,
      search: (prev: Record<string, unknown>) => ({
        ...prev,
        ...searchFromFilters(filters),
        page: undefined,
      }),
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const handleReset = () => {
    resetFilters()
    setIsOpen(false)
    navigate({
      to: fullUrlPath,
      search: (prev: Record<string, unknown>) => ({
        ...prev,
        wateringStatuses: undefined,
        regions: undefined,
        soil: undefined,
        hasCluster: undefined,
        plantingYears: undefined,
        page: undefined,
      }),
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const handleClose = () => {
    setIsOpen(false)
  }

  const handleOpen = () => {
    applyOldStateToTags(appliedFilters)
    setIsOpen(true)
  }

  const count = useMemo(
    () =>
      appliedFilters.statusTags.length +
      appliedFilters.regionTags.length +
      appliedFilters.soilTags.length +
      (appliedFilters.hasCluster !== undefined ? 1 : 0) +
      appliedFilters.plantingYears.length,
    [appliedFilters],
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

      <DialogRoot open={isOpen} onOpenChange={(open) => !open && handleClose()}>
        <DialogContent className="max-h-[80dvh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{headline}</DialogTitle>
          </DialogHeader>
          {children}

          <div className="flex flex-wrap gap-5 mt-6">
            <Button type="button" onClick={handleSubmit}>
              Anwenden
              <MoveRight className="icon-arrow-animate" />
            </Button>
            <Button variant="outline" onClick={handleReset}>
              Zurücksetzen
              <X />
            </Button>
          </div>
        </DialogContent>
      </DialogRoot>
    </div>
  )
}

export default Dialog
