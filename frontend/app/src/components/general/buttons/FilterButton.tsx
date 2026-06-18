import React from 'react'
import { SlidersHorizontal } from 'lucide-react'

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
  const active = activeCount > 0

  return (
    <button
      type="button"
      aria-label={ariaLabel}
      id="filter-button"
      aria-selected={active}
      onClick={onClick}
      className={`relative flex cursor-pointer items-center justify-center gap-x-2 rounded-full border border-green-light bg-white font-medium transition-colors duration-300 hover:bg-green-light-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-green-dark ${
        isOnMap
          ? 'z-[1000] px-5 py-2 shadow-cards'
          : 'h-10 w-10 sm:h-auto sm:w-auto sm:px-5 sm:py-2'
      } ${active ? 'bg-green-light-200' : ''}`}
    >
      {!isOnMap && <SlidersHorizontal className="size-5 sm:hidden" aria-hidden />}
      <span className={isOnMap ? '' : 'hidden sm:inline'}>Filter</span>
      <span
        className={`${isOnMap ? 'flex' : 'hidden sm:flex'} h-6 w-6 items-center justify-center rounded-full bg-green-dark/20 text-sm`}
      >
        {activeCount}
      </span>
      {!isOnMap && active && (
        <span
          className="absolute -right-0.5 -top-0.5 h-2.5 w-2.5 rounded-full bg-green-dark sm:hidden"
          aria-hidden
        />
      )}
    </button>
  )
}

export default FilterButton
