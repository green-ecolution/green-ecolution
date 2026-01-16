import React from 'react'
import { Plus } from 'lucide-react'
import SelectedCard from '../../cards/SelectedCard'
import { Button } from '@green-ecolution/ui'

interface SelectEntitiesProps {
  onDelete: (itemId: number) => void
  onChange?: (entries: number[]) => void // TODO: make not optinal after refactoring
  entityIds: number[]
  onAdd: () => void
  label: string
  type: 'tree' | 'cluster'
  required?: boolean
}

const SelectEntities: React.FC<SelectEntitiesProps> = ({
  onChange,
  entityIds,
  onAdd,
  label,
  type,
  required = false,
}) => {
  return (
    <div>
      <p className="block font-semibold text-dark-800 mb-2.5">
        Zugehörige {label}
        {required && <span className="text-red">&nbsp;*</span>}
      </p>

      <ul className="space-y-3">
        {entityIds.length === 0 ? (
          <li className="text-dark-600 font-semibold text-sm">
            {required ? (
              <p className="text-red">Es muss mindestens eine Auswahl getroffen werden.</p>
            ) : (
              <p>Hier können Sie zugehörige {label} verlinken.</p>
            )}
          </li>
        ) : (
          entityIds.map((entityId) => (
            <li key={entityId}>
              <SelectedCard
                type={type}
                id={entityId}
                onClick={(id) => {
                  onChange?.(entityIds.filter((i) => i !== id))
                }}
              />
            </li>
          ))
        )}
      </ul>

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
    </div>
  )
}

export default SelectEntities
