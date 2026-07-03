import React from 'react'
import { Plus } from 'lucide-react'
import SelectedCard from '../../cards/SelectedCard'
import { Button } from '@green-ecolution/ui'

interface SelectEntitiesProps {
  onChange: (entries: string[]) => void
  entityIds: string[]
  onAdd?: () => void
  label: string
  type: 'tree' | 'cluster'
  required?: boolean
  // Grow to fill the parent flex column and scroll the list internally, so a fixed-height
  // container (the map panel) never scrolls as a whole.
  fill?: boolean
  emptyHint?: string
}

const SelectEntities: React.FC<SelectEntitiesProps> = ({
  onChange,
  entityIds,
  onAdd,
  label,
  type,
  required = false,
  fill = false,
  emptyHint,
}) => {
  const hasEntities = entityIds.length > 0

  return (
    <div className={fill ? 'flex min-h-0 flex-1 flex-col' : undefined}>
      <div className="mb-2.5 flex items-center justify-between gap-2">
        <p className="block font-semibold text-dark-800">
          Zugehörige {label}
          {required && <span className="text-destructive">&nbsp;*</span>}
        </p>
        {hasEntities && (
          <span className="inline-flex min-w-6 items-center justify-center rounded-full bg-dark-100 px-2 py-0.5 text-xs font-semibold tabular-nums text-dark-600">
            {entityIds.length}
          </span>
        )}
      </div>

      {hasEntities ? (
        // Scroll only the selection so a long list never pushes the form's actions out of reach.
        <ul
          className={
            fill
              ? '-mx-1 min-h-0 flex-1 overflow-y-auto px-1'
              : '-mx-1 max-h-80 overflow-y-auto px-1'
          }
        >
          {entityIds.map((entityId) => (
            <li key={entityId}>
              <SelectedCard
                type={type}
                id={entityId}
                onClick={(id) => {
                  onChange(entityIds.filter((i) => i !== id))
                }}
              />
            </li>
          ))}
        </ul>
      ) : (
        <div className="rounded-lg border border-dashed border-dark-200 bg-dark-50/60 px-4 py-6 text-center text-sm">
          {required ? (
            <p className="font-semibold text-destructive">
              Es muss mindestens eine Auswahl getroffen werden.
            </p>
          ) : (
            <>
              <p className="font-medium text-dark-800">Noch keine {label} ausgewählt.</p>
              {emptyHint && <p className="mt-1 text-dark-600">{emptyHint}</p>}
            </>
          )}
        </div>
      )}

      {onAdd && (
        <Button
          type="button"
          variant="outline"
          onClick={(e) => {
            e.preventDefault()
            onAdd()
          }}
          className="mt-6"
        >
          {label} hinzufügen
          <Plus />
        </Button>
      )}
    </div>
  )
}

export default SelectEntities
