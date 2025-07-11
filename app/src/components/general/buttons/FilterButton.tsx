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
      className={`relative font-nunito-sans text-base ${isOnMap ? 'z-[1000] shadow-cards' : ''} bg-white border border-green-light px-5 py-2 font-medium rounded-full flex items-center gap-x-2 transition-colors ease-in-out duration-300 ${activeCount > 0 ? 'bg-green-light-200' : ''} hover:bg-green-light-200`}
      onClick={onClick}
    >
      Filter
      <span className="block bg-green-dark/20 w-6 h-6 rounded-full">{activeCount}</span>
    </button>
  )
}

export default FilterButton
