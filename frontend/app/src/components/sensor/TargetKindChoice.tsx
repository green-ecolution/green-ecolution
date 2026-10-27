import { ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { SensorTargetKind } from '@/lib/targets'

interface TargetKindChoiceProps {
  kinds: SensorTargetKind[]
  onChoose: (kind: SensorTargetKind) => void
}

const TargetKindChoice = ({ kinds, onChoose }: TargetKindChoiceProps) => {
  const { i18n } = useTranslation()
  return (
    <ul className="flex flex-col gap-2">
      {kinds.map((kind) => (
        <li key={`${kind.module}:${kind.target.kind}`}>
          <button
            type="button"
            onClick={() => onChoose(kind)}
            className="flex w-full items-center justify-between gap-3 rounded-xl border border-dark-100 px-4 py-3 text-left font-semibold transition-colors hover:bg-green-dark-50/50 focus-visible:outline-2 focus-visible:outline-green-dark"
          >
            {i18n.t(kind.target.labelKey)}
            <ChevronRight className="size-4 text-muted-foreground" aria-hidden />
          </button>
        </li>
      ))}
    </ul>
  )
}

export default TargetKindChoice
