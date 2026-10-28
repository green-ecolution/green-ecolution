import { useEffect, useEffectEvent, useState } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { FormField, Label, Switch } from '@green-ecolution/ui'
import type { StopOutcomeDto } from '@green-ecolution/backend-client'
import type { OutcomeInputProps } from '@/modules/types'
import { wasteBinQueries } from '../../api/queries'
import type { WasteBin } from '../../api/client'

interface BinOutcomeDraft {
  emptied: boolean
  amount: string
}

const UNTOUCHED: BinOutcomeDraft = { emptied: true, amount: '' }

// A bin that turns visible after a refetch has no entry yet.
const draftOf = (drafts: Record<string, BinOutcomeDraft>, id: string) => drafts[id] ?? UNTOUCHED

const parseAmount = (raw: string): number | undefined | null => {
  if (raw.trim() === '') return undefined
  const amount = Number(raw)
  return Number.isFinite(amount) && amount >= 0 ? amount : null
}

const isInvalidAmount = (raw: string) => parseAmount(raw) === null

const toOutcomes = (
  bins: WasteBin[],
  drafts: Record<string, BinOutcomeDraft>,
): { outcomes: StopOutcomeDto[]; valid: boolean } => {
  const outcomes: StopOutcomeDto[] = []
  for (const { id } of bins) {
    const { emptied, amount: raw } = draftOf(drafts, id)
    const amount = parseAmount(raw)
    if (amount === null) return { outcomes: [], valid: false }
    outcomes.push({ stop: id, completed: emptied, amount })
  }
  return { outcomes, valid: true }
}

const OutcomeFields = ({
  bins,
  onChange,
}: {
  bins: WasteBin[]
  onChange: OutcomeInputProps['onChange']
}) => {
  const { t } = useTranslation('waste')
  const [drafts, setDrafts] = useState<Record<string, BinOutcomeDraft>>({})
  const report = useEffectEvent((current: Record<string, BinOutcomeDraft>) => {
    const { outcomes, valid } = toOutcomes(bins, current)
    onChange(outcomes, valid)
  })
  useEffect(() => report(drafts), [drafts, bins])

  const update = (id: string, change: Partial<BinOutcomeDraft>) =>
    setDrafts((current) => ({ ...current, [id]: { ...draftOf(current, id), ...change } }))

  return (
    <fieldset className="mt-6">
      <legend className="mb-2.5 block font-semibold text-dark-800">
        {t('planning.outcome.legend')}
      </legend>
      <p className="-mt-2 mb-2.5 text-sm text-dark-600">{t('planning.outcome.hint')}</p>
      <ul className="flex flex-col">
        {bins.map((bin) => {
          const draft = draftOf(drafts, bin.id)
          const switchId = `bin-emptied-${bin.id}`
          return (
            <li
              key={bin.id}
              className="flex flex-wrap items-center justify-between gap-4 border-b border-dark-100 py-3 last:border-0"
            >
              <div className="flex min-w-0 items-center gap-3">
                <Switch
                  id={switchId}
                  checked={draft.emptied}
                  onCheckedChange={(emptied) => update(bin.id, { emptied })}
                  aria-label={t('planning.outcome.emptiedLabel', { number: bin.number })}
                />
                <Label htmlFor={switchId} className="min-w-0 cursor-pointer">
                  <span className="block truncate font-medium tabular-nums text-dark">
                    {bin.number}
                  </span>
                  <span className="block text-xs font-normal tabular-nums text-dark-600">
                    {t('units.liters', { value: bin.volumeLiters })}
                  </span>
                </Label>
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <FormField
                  type="number"
                  min={0}
                  label={t('planning.outcome.amountLabel', { number: bin.number })}
                  placeholder={t('planning.outcome.amountPlaceholder')}
                  value={draft.amount}
                  onChange={(event) => update(bin.id, { amount: event.target.value })}
                  error={
                    isInvalidAmount(draft.amount)
                      ? t('planning.outcome.negativeAmountError')
                      : undefined
                  }
                  className="max-w-36"
                  hideLabel
                />
                <span className="text-sm text-dark-600">
                  {t('planning.outcome.litersUnitLabel')}
                </span>
              </div>
            </li>
          )
        })}
      </ul>
    </fieldset>
  )
}

const BinOutcomeInput = ({ plan, onChange }: OutcomeInputProps) => {
  const { data: bins } = useSuspenseQuery(wasteBinQueries.byIds(plan.stops))
  return <OutcomeFields key={plan.id} bins={bins} onChange={onChange} />
}

export default BinOutcomeInput
