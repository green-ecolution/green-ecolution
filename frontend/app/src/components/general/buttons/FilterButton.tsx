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
      className={`cursor-pointer font-medium rounded-full flex items-center gap-x-2 px-5 py-2 transition-colors duration-300 ${isOnMap ? 'z-[1000] shadow-cards' : ''} hover:bg-green-light-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-ring`}
      style={{
        backgroundColor: activeCount > 0 ? '#e8f0c8' : '#ffffff',
        borderWidth: '1px',
        borderStyle: 'solid',
        borderColor: '#acb63b',
        color: '#333333',
      }}
      onClick={onClick}
    >
      Filter
      <span
        className="flex items-center justify-center w-6 h-6 rounded-full text-sm"
        style={{ backgroundColor: 'color-mix(in srgb, var(--green-dark) 20%, transparent)' }}
      >
        {activeCount}
      </span>
    </button>
  )
}

export default FilterButton
