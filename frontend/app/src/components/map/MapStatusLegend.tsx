import { useEffect, useRef, useState } from 'react'
import { WateringStatus } from '@green-ecolution/backend-client'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

const AMPEL_STATUSES = [WateringStatus.Bad, WateringStatus.Moderate, WateringStatus.Good]
const NEUTRAL_STATUSES = [WateringStatus.JustWatered, WateringStatus.Unknown]

const LegendRow = ({ status }: { status: WateringStatus }) => {
  const { label, colorHex } = getWateringStatusDetails(status)
  return (
    <li className="flex items-center gap-2">
      <span
        aria-hidden
        className="size-3 shrink-0 rounded-full"
        style={{ backgroundColor: colorHex }}
      />
      <span className="text-sm text-dark-800">{label}</span>
    </li>
  )
}

const MapStatusLegend = () => {
  const [open, setOpen] = useState(false)
  const rootRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const onPointerDown = (e: PointerEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('pointerdown', onPointerDown)
    return () => document.removeEventListener('pointerdown', onPointerDown)
  }, [open])

  return (
    <div
      ref={rootRef}
      className="absolute bottom-6 left-4 z-[1000] flex flex-col items-start gap-2 lg:bottom-10 lg:left-10"
    >
      {open && (
        <div
          role="region"
          aria-label="Legende Bewässerungsstatus"
          className="w-52 rounded-xl bg-white p-4 shadow-cards"
        >
          <p className="text-sm font-semibold text-dark-800">Bewässerungsstatus</p>
          <ul className="mt-2 flex flex-col gap-1.5">
            {AMPEL_STATUSES.map((status) => (
              <LegendRow key={status} status={status} />
            ))}
          </ul>
          <div aria-hidden className="my-2 h-px bg-dark-800/10" />
          <ul className="flex flex-col gap-1.5">
            {NEUTRAL_STATUSES.map((status) => (
              <LegendRow key={status} status={status} />
            ))}
          </ul>
        </div>
      )}
      <button
        type="button"
        aria-expanded={open}
        aria-label={open ? 'Legende ausblenden' : 'Legende anzeigen'}
        onClick={() => setOpen((v) => !v)}
        className="grid size-9 place-items-center rounded-full bg-white shadow-cards"
      >
        <span aria-hidden className="text-base font-bold text-dark-800">
          ?
        </span>
      </button>
    </div>
  )
}

export default MapStatusLegend
