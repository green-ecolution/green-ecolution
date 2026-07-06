import { Input, Button } from '@green-ecolution/ui'
import { Search, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

interface SensorTreeSearchInputProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
}

const DEBOUNCE_MS = 300

const SensorTreeSearchInput = ({
  value,
  onChange,
  placeholder = 'Baumnummer oder Baumart suchen…',
}: SensorTreeSearchInputProps) => {
  const [local, setLocal] = useState(value)
  const [syncedValue, setSyncedValue] = useState(value)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Mirror the controlled value into local state when it changes externally,
  // without an effect (see React "You Might Not Need an Effect").
  if (value !== syncedValue) {
    setSyncedValue(value)
    setLocal(value)
  }

  useEffect(
    () => () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current)
    },
    [],
  )

  const schedule = (next: string) => {
    setLocal(next)
    if (timeoutRef.current) clearTimeout(timeoutRef.current)
    timeoutRef.current = setTimeout(() => onChange(next), DEBOUNCE_MS)
  }

  return (
    <div className="relative flex items-center">
      <Search className="pointer-events-none absolute left-3 size-4 text-dark-500" aria-hidden />
      <Input
        role="searchbox"
        value={local}
        onChange={(e) => schedule(e.target.value)}
        placeholder={placeholder}
        className="pl-9 pr-9 text-base"
        autoComplete="off"
        inputMode="search"
      />
      {local.length > 0 && (
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="Leeren"
          onClick={() => schedule('')}
          className="absolute right-1 size-7"
        >
          <X className="size-4" />
        </Button>
      )}
    </div>
  )
}

export default SensorTreeSearchInput
export type { SensorTreeSearchInputProps }
