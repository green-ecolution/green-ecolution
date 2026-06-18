import { X } from 'lucide-react'

interface SelectedTagListProps {
  options: { value: string; label: string }[]
  value: string[]
  onRemove: (value: string) => void
}

const SelectedTagList = ({ options, value, onRemove }: SelectedTagListProps) => {
  if (value.length === 0) return null

  const labelFor = (v: string) => options.find((o) => o.value === v)?.label ?? v

  return (
    <ul className="mt-2 flex flex-wrap gap-1.5">
      {value.map((v) => (
        <li key={v}>
          <button
            type="button"
            onClick={() => onRemove(v)}
            aria-label={`${labelFor(v)} entfernen`}
            className="inline-flex items-center gap-1 rounded-full border border-dark-200 bg-dark-50 py-0.5 pl-2.5 pr-1.5 text-xs text-dark-700 transition hover:border-green-dark"
          >
            {labelFor(v)}
            <X className="h-3 w-3 text-dark-500" aria-hidden />
          </button>
        </li>
      ))}
    </ul>
  )
}

export default SelectedTagList
