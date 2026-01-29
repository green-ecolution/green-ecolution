import { Tree } from '@green-ecolution/backend-client'
import { memo, useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { treeQuery } from '@/api/queries'
import FilterProvider, { useFilter, Filters } from '@/context/FilterContext'
import FilterButton from '@/components/general/buttons/FilterButton'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import ClusterFieldset from '@/components/general/filter/fieldsets/ClusterFieldset'
import PlantingYearFieldset from '@/components/general/filter/fieldsets/PlantingYearFieldset'
import useMapInteractions from '@/hooks/useMapInteractions'
import WithAllTrees from './WithAllTrees'
import WithFilterdTrees from './WithFilterdTrees'
import {
  Button,
  Dialog as DialogRoot,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'

export interface WithFilterableTreesProps {
  onClick?: (tree: Tree) => void
  selectedTrees?: number[]
  hasHighlightedTree?: number
}

interface AppliedFilters {
  wateringStatuses?: string[]
  hasCluster?: boolean
  plantingYears?: number[]
}

interface FilterableTreesContentProps extends WithFilterableTreesProps {
  appliedFilters: AppliedFilters
}

const FilterableTreesContent = memo(
  ({ onClick, selectedTrees, hasHighlightedTree, appliedFilters }: FilterableTreesContentProps) => {
    const hasActiveFilter = useMemo(
      () =>
        (appliedFilters.wateringStatuses?.length ?? 0) > 0 ||
        appliedFilters.hasCluster !== undefined ||
        (appliedFilters.plantingYears?.length ?? 0) > 0,
      [appliedFilters.wateringStatuses, appliedFilters.hasCluster, appliedFilters.plantingYears],
    )

    const { data: treesRes } = useQuery({
      enabled: hasActiveFilter,
      ...treeQuery({
        wateringStatuses: appliedFilters.wateringStatuses,
        hasCluster: appliedFilters.hasCluster,
        plantingYears: appliedFilters.plantingYears,
      }),
    })

    if (hasActiveFilter) {
      return (
        <WithFilterdTrees
          onClick={onClick}
          selectedTrees={selectedTrees}
          hasHighlightedTree={hasHighlightedTree}
          filterdTrees={treesRes?.data ?? []}
        />
      )
    }

    return (
      <WithAllTrees
        onClick={onClick}
        selectedTrees={selectedTrees}
        hasHighlightedTree={hasHighlightedTree}
      />
    )
  },
)

FilterableTreesContent.displayName = 'FilterableTreesContent'

const FilterableTreesInner = memo(
  ({ onClick, selectedTrees, hasHighlightedTree }: WithFilterableTreesProps) => {
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
        hasCluster: filters.hasCluster,
        plantingYears: filters.plantingYears.length > 0 ? filters.plantingYears : undefined,
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
      () =>
        (appliedFilters.wateringStatuses?.length ?? 0) +
        (appliedFilters.hasCluster !== undefined ? 1 : 0) +
        (appliedFilters.plantingYears?.length ?? 0),
      [appliedFilters],
    )

    return (
      <>
        <div className="absolute top-6 left-4 z-[1000]">
          <div className="font-nunito-sans text-base">
            <FilterButton
              activeCount={filterCount}
              ariaLabel="Bäume filtern"
              isOnMap
              onClick={() => (isOpen ? handleClose() : handleOpen())}
            />

            <DialogRoot open={isOpen} onOpenChange={(open) => !open && handleClose()}>
              <DialogContent className="max-h-[80dvh] overflow-y-auto">
                <DialogHeader>
                  <DialogTitle>Bäume filtern</DialogTitle>
                </DialogHeader>
                <StatusFieldset />
                <ClusterFieldset />
                <PlantingYearFieldset />

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
        <FilterableTreesContent
          onClick={onClick}
          selectedTrees={selectedTrees}
          hasHighlightedTree={hasHighlightedTree}
          appliedFilters={appliedFilters}
        />
      </>
    )
  },
)

FilterableTreesInner.displayName = 'FilterableTreesInner'

const WithFilterableTrees = memo((props: WithFilterableTreesProps) => {
  return (
    <FilterProvider>
      <FilterableTreesInner {...props} />
    </FilterProvider>
  )
})

WithFilterableTrees.displayName = 'WithFilterableTrees'

export default WithFilterableTrees
