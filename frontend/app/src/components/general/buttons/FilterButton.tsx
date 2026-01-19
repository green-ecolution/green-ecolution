import React from 'react'

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
    <button
      type="button"
      aria-label={ariaLabel}
      id="filter-button"
      aria-selected={activeCount > 0}
      className={`cursor-pointer font-medium rounded-full flex items-center gap-x-2 px-5 py-2 transition-colors duration-300 bg-white border border-green-light ${isOnMap ? 'z-[1000] shadow-cards' : ''} ${activeCount > 0 ? 'bg-green-light-200' : ''} hover:bg-green-light-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-green-dark`}
      onClick={onClick}
    >
      Filter
      <span className="flex items-center justify-center w-6 h-6 rounded-full text-sm bg-green-dark/20">
        {activeCount}
      </span>
    </button>
  )
}

export default FilterButton
