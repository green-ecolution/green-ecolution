import { TreeClusterInList } from '@green-ecolution/backend-client'
import { memo, useCallback, useDeferredValue, useMemo, useState } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { treeClusterQuery } from '@/api/queries'
import FilterProvider, { useFilter, Filters } from '@/context/FilterContext'
import FilterButton from '@/components/general/buttons/FilterButton'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import RegionFieldset from '@/components/general/filter/fieldsets/RegionFieldset'
import useMapInteractions from '@/hooks/useMapInteractions'
import MarkerList from './MarkerList'
import { ClusterIcon } from '../MapMarker'
import { getStatusColor } from '../utils'
import {
  Button,
  Dialog as DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'

const defaultHighlighted: number[] = []
const defaultDisabled: number[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

export interface WithFilterableClustersProps {
  onClick?: (cluster: TreeClusterInList) => void
  highlightedClusters?: number[]
  disabledClusters?: number[]
}

interface AppliedFilters {
  wateringStatuses?: string[]
  regions?: string[]
}

interface FilterableClustersContentProps extends WithFilterableClustersProps {
  appliedFilters: AppliedFilters
}

const FilterableClustersContent = memo(
  ({
    onClick,
    highlightedClusters = defaultHighlighted,
    disabledClusters = defaultDisabled,
    appliedFilters,
  }: FilterableClustersContentProps) => {
    const hasActiveFilter = useMemo(
      () =>
        (appliedFilters.wateringStatuses?.length ?? 0) > 0 ||
        (appliedFilters.regions?.length ?? 0) > 0,
      [appliedFilters.wateringStatuses, appliedFilters.regions],
    )

    const { data } = useSuspenseQuery(
      treeClusterQuery(
        hasActiveFilter
          ? {
              wateringStatuses: appliedFilters.wateringStatuses,
              regions: appliedFilters.regions,
            }
          : undefined,
      ),
    )

    const filteredData = useMemo(
      () =>
        data.data.filter(
          (cluster) =>
            cluster.latitude !== null &&
            cluster.longitude !== null &&
            cluster.treeIds !== undefined,
        ),
      [data.data],
    )
    const deferredData = useDeferredValue(filteredData)

    const highlightedSet = useMemo(() => new Set(highlightedClusters), [highlightedClusters])
    const disabledSet = useMemo(() => new Set(disabledClusters), [disabledClusters])

    const getIcon = useCallback(
      (c: TreeClusterInList) =>
        ClusterIcon(
          getStatusColor(c.wateringStatus),
          highlightedSet.has(c.id),
          disabledSet.has(c.id),
          c.treeIds?.length ?? 0,
        ),
      [highlightedSet, disabledSet],
    )

    const getId = useCallback((c: TreeClusterInList) => c.id, [])
    const getTooltip = useCallback((c: TreeClusterInList) => c.name, [])

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
  },
)

FilterableClustersContent.displayName = 'FilterableClustersContent'

const FilterableClustersInner = memo(
  ({ onClick, highlightedClusters, disabledClusters }: WithFilterableClustersProps) => {
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

    const handleMapInteractions = useCallback(
      (isOpen: boolean) => {
        if (isOpen) {
          disableDragging()
        } else {
          enableDragging()
        }
      },
      [disableDragging, enableDragging],
    )

    const handleOpen = useCallback(() => {
      setOldValues(filters)
      setIsOpen(true)
      handleMapInteractions(true)
    }, [filters, handleMapInteractions])

    const handleClose = useCallback(() => {
      setIsOpen(false)
      applyOldStateToTags(oldValues)
      handleMapInteractions(false)
    }, [applyOldStateToTags, oldValues, handleMapInteractions])

    const handleSubmit = useCallback(() => {
      setAppliedFilters({
        wateringStatuses: filters.statusTags.length > 0 ? filters.statusTags : undefined,
        regions: filters.regionTags.length > 0 ? filters.regionTags : undefined,
      })
      setIsOpen(false)
      handleMapInteractions(false)
    }, [filters, handleMapInteractions])

    const handleReset = useCallback(() => {
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
    }, [applyOldStateToTags, resetFilters, handleMapInteractions])

    const filterCount = useMemo(
      () => (appliedFilters.wateringStatuses?.length ?? 0) + (appliedFilters.regions?.length ?? 0),
      [appliedFilters],
    )

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
  },
)

FilterableClustersInner.displayName = 'FilterableClustersInner'

const WithFilterableClusters = memo((props: WithFilterableClustersProps) => {
  return (
    <FilterProvider>
      <FilterableClustersInner {...props} />
    </FilterProvider>
  )
})

WithFilterableClusters.displayName = 'WithFilterableClusters'

export default WithFilterableClusters
