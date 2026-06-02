import type { TreeClusterInList } from '@/api/backendApi'
import { useDeferredValue, useState } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { treeClusterQuery } from '@/api/queries'
import FilterProvider, { useFilter, Filters } from '@/context/FilterContext'
import FilterButton from '@/components/general/buttons/FilterButton'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import RegionFieldset from '@/components/general/filter/fieldsets/RegionFieldset'
import useMapInteractions from '@/hooks/useMapInteractions'
import MarkerList from './MarkerList'
import { ClusterIcon } from '../markerIcons'
import { getStatusColor } from '../utils'
import {
  Button,
  Dialog as DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'

const defaultHighlighted: string[] = []
const defaultDisabled: string[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

const getId = (c: TreeClusterInList) => c.id
const getTooltip = (c: TreeClusterInList) => c.name

export interface WithFilterableClustersProps {
  onClick?: (cluster: TreeClusterInList) => void
  highlightedClusters?: string[]
  disabledClusters?: string[]
}

interface AppliedFilters {
  wateringStatuses?: string[]
  regions?: string[]
}

interface FilterableClustersContentProps extends WithFilterableClustersProps {
  appliedFilters: AppliedFilters
}

const FilterableClustersContent = ({
  onClick,
  highlightedClusters = defaultHighlighted,
  disabledClusters = defaultDisabled,
  appliedFilters,
}: FilterableClustersContentProps) => {
  const hasActiveFilter =
    (appliedFilters.wateringStatuses?.length ?? 0) > 0 || (appliedFilters.regions?.length ?? 0) > 0

  // TODO: wateringStatuses and regions filter params are not yet supported in the new API
  const { data } = useSuspenseQuery(treeClusterQuery(hasActiveFilter ? {} : undefined))

  const filteredData = data.data.filter(
    (cluster) =>
      cluster.latitude !== null && cluster.longitude !== null && cluster.treeIds !== undefined,
  )
  const deferredData = useDeferredValue(filteredData)

  const highlightedSet = new Set(highlightedClusters)
  const disabledSet = new Set(disabledClusters)

  const getIcon = (c: TreeClusterInList) =>
    ClusterIcon(
      getStatusColor(c.wateringStatus),
      highlightedSet.has(c.id),
      disabledSet.has(c.id),
      c.treeIds?.length ?? 0,
    )

  return (
    <MarkerList
      data={deferredData}
      onClick={onClick}
      icon={getIcon}
      getId={getId}
      tooltipContent={getTooltip}
      tooltipOptions={tooltipOptions}
    />
  )
}

FilterableClustersContent.displayName = 'FilterableClustersContent'

const FilterableClustersInner = ({
  onClick,
  highlightedClusters,
  disabledClusters,
}: WithFilterableClustersProps) => {
  const { enableDragging, disableDragging } = useMapInteractions()
  const { filters, resetFilters, applyOldStateToTags } = useFilter()
  const [isOpen, setIsOpen] = useState(false)
  const [oldValues, setOldValues] = useState<Filters>({
    statusTags: [],
    regionTags: [],
    hasCluster: undefined,
    plantingYears: [],
  })
  const [appliedFilters, setAppliedFilters] = useState<AppliedFilters>({})

  const handleMapInteractions = (isOpen: boolean) => {
    if (isOpen) {
      disableDragging()
    } else {
      enableDragging()
    }
  }

  const handleOpen = () => {
    setOldValues(filters)
    setIsOpen(true)
    handleMapInteractions(true)
  }

  const handleClose = () => {
    setIsOpen(false)
    applyOldStateToTags(oldValues)
    handleMapInteractions(false)
  }

  const handleSubmit = () => {
    setAppliedFilters({
      wateringStatuses: filters.statusTags.length > 0 ? filters.statusTags : undefined,
      regions: filters.regionTags.length > 0 ? filters.regionTags : undefined,
    })
    setIsOpen(false)
    handleMapInteractions(false)
  }

  const handleReset = () => {
    applyOldStateToTags({
      statusTags: [],
      regionTags: [],
      hasCluster: undefined,
      plantingYears: [],
    })
    resetFilters()
    setAppliedFilters({})
    setIsOpen(false)
    handleMapInteractions(false)
  }

  const filterCount =
    (appliedFilters.wateringStatuses?.length ?? 0) + (appliedFilters.regions?.length ?? 0)

  return (
    <>
      <div className="absolute top-6 left-4 z-[1000]">
        <div className="font-nunito-sans text-base">
          <FilterButton
            activeCount={filterCount}
            ariaLabel="Bewässerungsgruppen filtern"
            isOnMap
            onClick={() => (isOpen ? handleClose() : handleOpen())}
          />

          <DialogRoot open={isOpen} onOpenChange={(open) => !open && handleClose()}>
            <DialogContent className="max-h-[80dvh] overflow-y-auto">
              <DialogHeader>
                <DialogTitle>Bewässerungsgruppen filtern</DialogTitle>
              </DialogHeader>
              <StatusFieldset />
              <RegionFieldset />

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
      </div>
      <FilterableClustersContent
        onClick={onClick}
        highlightedClusters={highlightedClusters}
        disabledClusters={disabledClusters}
        appliedFilters={appliedFilters}
      />
    </>
  )
}

FilterableClustersInner.displayName = 'FilterableClustersInner'

const WithFilterableClusters = (props: WithFilterableClustersProps) => {
  return (
    <FilterProvider>
      <FilterableClustersInner {...props} />
    </FilterProvider>
  )
}

WithFilterableClusters.displayName = 'WithFilterableClusters'

export default WithFilterableClusters
