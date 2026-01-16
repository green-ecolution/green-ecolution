import React from 'react'
import { Button } from '@green-ecolution/ui'

interface FilterButtonProps {
  ariaLabel: string
  activeCount: number
  isOnMap: boolean
  onClick: () => void
}

const FilterButton: React.FC<FilterButtonProps> = ({
  ariaLabel,
  activeCount,
  onClick,
  isOnMap,
}) => {
  return (
    <Button
      variant="outline"
      aria-label={ariaLabel}
      id="filter-button"
      aria-selected={activeCount > 0}
      className={`rounded-full border-green-light ${isOnMap ? 'z-[1000] shadow-cards' : ''} ${activeCount > 0 ? 'bg-green-light-200' : 'bg-white'} hover:bg-green-light-200`}
      onClick={onClick}
    >
      Filter
      <span className="block bg-green-dark/20 w-6 h-6 rounded-full text-sm">{activeCount}</span>
    </Button>
  )
}

export default FilterButton
