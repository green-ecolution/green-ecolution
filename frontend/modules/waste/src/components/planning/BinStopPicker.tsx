import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { ListChecks, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FormField,
  ListCard,
  ListCardActions,
  ListCardContent,
} from '@green-ecolution/ui'
import type { StopPickerProps } from '@/modules/types'
import { keepUnresolvedStops } from '@/modules/planning'
import { wasteBinQueries } from '../../api/queries'
import type { WasteBin } from '../../api/client'
import FillStateBadge from '../FillStateBadge'

const BinStopPicker = ({ value, onChange, error }: StopPickerProps) => {
  const { t } = useTranslation(['waste', 'common'])
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const binsQuery = useQuery(wasteBinQueries.markers())
  const bins = binsQuery.data

  const visible = new Map(bins?.map((bin) => [bin.id, bin]))
  // Before the first answer any stop may be hidden, so none is listed yet.
  const shown = bins ? value.filter((id) => visible.has(id)) : []
  const shownBins = shown.flatMap((id) => visible.get(id) ?? [])
  const report = (next: string[]) => onChange(keepUnresolvedStops(next, value, shown))

  const toggle = (id: string, selected: boolean) =>
    report(selected ? [...shown, id] : shown.filter((stop) => stop !== id))

  const selectFull = () => {
    const full = (bins ?? [])
      .filter((bin) => bin.fillState === 'full' && !shown.includes(bin.id))
      .map(({ id }) => id)
    report([...shown, ...full])
  }

  const needle = query.trim().toLowerCase()
  const matching = (bins ?? []).filter((bin) => bin.number.toLowerCase().includes(needle))
  const label = t('planning.stopPicker.label')

  return (
    <div>
      <div className="mb-2.5 flex items-center justify-between gap-2">
        <p className="font-semibold text-dark-800">
          {t('common:form.selectEntities.selectedLabel', { label })}
        </p>
        {shown.length > 0 && (
          <span className="inline-flex min-w-6 items-center justify-center rounded-full bg-dark-100 px-2 py-0.5 text-xs font-semibold tabular-nums text-dark-600">
            {shown.length}
          </span>
        )}
      </div>

      {shownBins.length > 0 ? (
        <ul className="-mx-1 max-h-80 overflow-y-auto px-1">
          {shownBins.map((bin) => (
            <li key={bin.id}>
              <SelectedBin bin={bin} onRemove={() => toggle(bin.id, false)} />
            </li>
          ))}
        </ul>
      ) : (
        <div className="rounded-lg border border-dashed border-dark-200 bg-dark-50/60 px-4 py-6 text-center text-sm font-medium text-dark-800">
          {t('common:form.selectEntities.emptyLabel', { label })}
        </div>
      )}

      <Button type="button" variant="outline" onClick={() => setOpen(true)} className="mt-6">
        {t('planning.stopPicker.openButton')}
        <ListChecks />
      </Button>

      {error && (
        <p role="alert" aria-live="assertive" className="mt-2 text-sm text-destructive">
          {t('planning.stopPicker.requiredError')}
        </p>
      )}

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="flex max-h-[calc(100dvh-3rem)] max-w-xl flex-col">
          <DialogHeader>
            <DialogTitle>{t('planning.stopPicker.dialogTitle')}</DialogTitle>
            <DialogDescription>{t('planning.stopPicker.dialogDescription')}</DialogDescription>
          </DialogHeader>

          <div className="flex flex-wrap items-end gap-3">
            <FormField
              type="search"
              label={t('planning.stopPicker.searchLabel')}
              placeholder={t('planning.stopPicker.searchPlaceholder')}
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              className="min-w-48 flex-1"
              hideLabel
            />
            <Button type="button" variant="outline" onClick={selectFull} disabled={!bins}>
              {t('planning.stopPicker.selectFull')}
            </Button>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto">
            {binsQuery.isError && (
              <p className="py-6 text-center text-sm text-destructive">
                {t('planning.stopPicker.loadError')}
              </p>
            )}
            {bins && matching.length === 0 && (
              <p className="py-6 text-center text-sm text-dark-600">
                {bins.length === 0
                  ? t('planning.stopPicker.noBins')
                  : t('planning.stopPicker.noMatch')}
              </p>
            )}
            <ul className="flex flex-col">
              {matching.map((bin) => (
                <li key={bin.id}>
                  <BinOption
                    bin={bin}
                    selected={shown.includes(bin.id)}
                    onSelectedChange={(selected) => toggle(bin.id, selected)}
                  />
                </li>
              ))}
            </ul>
          </div>

          <DialogFooter>
            <Button type="button" onClick={() => setOpen(false)}>
              {t('planning.stopPicker.done')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

const SelectedBin = ({ bin, onRemove }: { bin: WasteBin; onRemove: () => void }) => {
  const { t } = useTranslation('waste')
  return (
    <ListCard size="compact" hoverable={false} className="mb-3">
      <ListCardContent>
        <div className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1">
          <span className="font-semibold tabular-nums text-dark">{bin.number}</span>
          <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
          <span className="text-xs tabular-nums text-dark-600">
            {t('units.liters', { value: bin.volumeLiters })}
          </span>
        </div>
      </ListCardContent>
      <ListCardActions>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="text-dark-600 hover:text-destructive"
          onClick={onRemove}
          aria-label={t('planning.stopPicker.removeAriaLabel', { number: bin.number })}
        >
          <X className="size-5" />
        </Button>
      </ListCardActions>
    </ListCard>
  )
}

const BinOption = ({
  bin,
  selected,
  onSelectedChange,
}: {
  bin: WasteBin
  selected: boolean
  onSelectedChange: (selected: boolean) => void
}) => {
  const { t } = useTranslation('waste')
  const id = `bin-option-${bin.id}`
  return (
    <div className="flex items-center gap-3 border-b border-dark-100 py-2.5 last:border-0">
      <Checkbox
        id={id}
        checked={selected}
        onCheckedChange={(checked) => onSelectedChange(checked === true)}
        aria-label={t('planning.stopPicker.selectAriaLabel', { number: bin.number })}
      />
      <label
        htmlFor={id}
        className="flex min-w-0 flex-1 cursor-pointer flex-wrap items-center gap-x-3 gap-y-1"
      >
        <span className="font-medium tabular-nums text-dark">{bin.number}</span>
        <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
        <span className="text-xs tabular-nums text-dark-600">
          {t('units.liters', { value: bin.volumeLiters })}
        </span>
      </label>
    </div>
  )
}

export default BinStopPicker
