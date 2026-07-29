import { useId } from 'react'
import {
  Switch,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@green-ecolution/ui'

interface PermissionToggleProps {
  label: string
  hint: string
  checked: boolean
  disabled: boolean
  disabledReason?: string
  onCheckedChange: () => void
}

const PermissionToggle = ({
  label,
  hint,
  checked,
  disabled,
  disabledReason,
  onCheckedChange,
}: PermissionToggleProps) => {
  const hintId = useId()

  const control = (
    <Switch
      checked={checked}
      disabled={disabled}
      onCheckedChange={onCheckedChange}
      aria-label={label}
      aria-describedby={hintId}
    />
  )

  return (
    <div className="flex items-center justify-between gap-4 border-t border-dark-50 px-5 py-4 first:border-t-0">
      <div className="min-w-0">
        <p className="font-nunito-sans text-sm font-semibold text-dark">{label}</p>
        <p id={hintId} className="mt-0.5 text-sm text-dark-600">
          {hint}
        </p>
      </div>
      {disabled && disabledReason ? (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="shrink-0">{control}</span>
            </TooltipTrigger>
            <TooltipContent>{disabledReason}</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      ) : (
        <span className="shrink-0">{control}</span>
      )}
    </div>
  )
}

export default PermissionToggle
