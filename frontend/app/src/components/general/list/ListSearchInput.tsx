import { useEffect, useRef, useState } from 'react'
import { Search, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@green-ecolution/ui'

interface ListSearchInputProps {
  value: string
  onChange: (value: string) => void
  label: string
  placeholder?: string
  debounceMs?: number
}

const ListSearchInput = ({
  value,
  onChange,
  label,
  placeholder,
  debounceMs = 300,
}: ListSearchInputProps) => {
  const { t } = useTranslation('common')
  const [draft, setDraft] = useState(value)
  const applied = useRef(value)

  useEffect(() => {
    if (value === applied.current) return
    applied.current = value
    // eslint-disable-next-line react-x/set-state-in-effect -- adopts a search term the parent changed on its own, e.g. a filter reset
    setDraft(value)
  }, [value])

  useEffect(() => {
    const trimmed = draft.trim()
    if (trimmed === value) return
    const timeout = setTimeout(() => {
      applied.current = trimmed
      onChange(trimmed)
    }, debounceMs)
    return () => clearTimeout(timeout)
  }, [draft, value, debounceMs, onChange])

  return (
    <InputGroup>
      <InputGroupAddon align="inline-start">
        <Search aria-hidden className="size-4 text-dark-600" />
      </InputGroupAddon>
      <InputGroupInput
        type="search"
        aria-label={label}
        placeholder={placeholder}
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
      />
      {draft.length > 0 && (
        <InputGroupAddon align="inline-end">
          <button
            type="button"
            onClick={() => setDraft('')}
            aria-label={t('list.clearSearch')}
            className="cursor-pointer rounded-full p-1 text-dark-600 transition-[color,opacity] duration-quick ease-out hover:text-green-dark active:opacity-70 motion-reduce:transition-none"
          >
            <X aria-hidden className="size-4" />
          </button>
        </InputGroupAddon>
      )}
    </InputGroup>
  )
}

export default ListSearchInput
