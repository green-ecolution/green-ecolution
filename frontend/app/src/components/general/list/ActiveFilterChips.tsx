import type { ReactNode } from 'react'
import { RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { FilterChip } from '@green-ecolution/ui'

export interface FilterChipDescriptor {
  id: string
  label: string
  onRemove: () => void
}

interface ActiveFilterChipsProps {
  chips: FilterChipDescriptor[]
  onReset: () => void
  resultLabel: string
  /** Chips a module renders itself, after the descriptor chips. */
  children?: ReactNode
  /** Whether `children` holds a chip, which decides about the reset link. */
  active?: boolean
}

const ActiveFilterChips = ({
  chips,
  onReset,
  resultLabel,
  children,
  active = false,
}: ActiveFilterChipsProps) => {
  const { t } = useTranslation('common')

  return (
    <div className="mt-4 flex flex-wrap items-center gap-2">
      {chips.map((chip) => (
        <FilterChip key={chip.id} label={chip.label} onRemove={chip.onRemove} />
      ))}
      {children}
      {(chips.length > 0 || active) && (
        <button
          type="button"
          onClick={onReset}
          className="flex cursor-pointer items-center gap-1.5 text-sm text-dark-600 underline transition-[color,opacity] duration-quick ease-out hover:text-green-dark active:opacity-70 motion-reduce:transition-none"
        >
          <RotateCcw aria-hidden className="size-3.5" />
          {t('list.resetFilters')}
        </button>
      )}
      <p aria-live="polite" className="ml-auto text-sm text-dark-600">
        {resultLabel}
      </p>
    </div>
  )
}

export default ActiveFilterChips
