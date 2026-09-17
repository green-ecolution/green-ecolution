import { useEffect, useState } from 'react'
import { Search, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Input, InputGroup, InputGroupAddon } from '@green-ecolution/ui'

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

  useEffect(() => {
    const trimmed = draft.trim()
    if (trimmed === value) return
    const timeout = setTimeout(() => onChange(trimmed), debounceMs)
    return () => clearTimeout(timeout)
  }, [draft, value, debounceMs, onChange])

  return (
    <InputGroup className="w-full sm:max-w-80">
      <InputGroupAddon align="inline-start">
        <Search aria-hidden className="size-4 text-dark-600" />
      </InputGroupAddon>
      <Input
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
            className="rounded-full p-1 text-dark-600 transition-colors hover:text-green-dark"
          >
            <X aria-hidden className="size-4" />
          </button>
        </InputGroupAddon>
      )}
    </InputGroup>
  )
}

export default ListSearchInput
