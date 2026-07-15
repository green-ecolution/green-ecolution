import { ChangeEvent, useEffect, useState } from 'react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
} from '@green-ecolution/ui'
import { SoilCondition } from '@green-ecolution/backend-client'
import { soilConditionLabel } from '@/hooks/details/useDetailsForSoilCondition'
import {
  SoilFractions,
  balanceFractions,
  classifySoilTexture,
  regionMidpoint,
} from '@/lib/soilTexture'
import SoilTextureTriangle from './SoilTextureTriangle'

const NEUTRAL_FRACTIONS: SoilFractions = { sand: 33, silt: 34, clay: 33 }

const FRACTION_FIELDS: { key: keyof SoilFractions; label: string }[] = [
  { key: 'sand', label: 'Sand' },
  { key: 'silt', label: 'Schluff' },
  { key: 'clay', label: 'Ton' },
]

interface SoilTextureDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  initialCondition: SoilCondition
  onApply: (condition: SoilCondition) => void
}

const SoilTextureDialog = ({
  open,
  onOpenChange,
  initialCondition,
  onApply,
}: SoilTextureDialogProps) => {
  const [fractions, setFractions] = useState<SoilFractions>(NEUTRAL_FRACTIONS)

  useEffect(() => {
    if (open) setFractions(regionMidpoint(initialCondition) ?? NEUTRAL_FRACTIONS)
  }, [open, initialCondition])

  const condition = classifySoilTexture(fractions.silt, fractions.clay)

  const handleChange = (field: keyof SoilFractions) => (event: ChangeEvent<HTMLInputElement>) => {
    setFractions((current) => balanceFractions(current, field, event.target.valueAsNumber))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Bodenart bestimmen</DialogTitle>
          <DialogDescription>
            Gib die Korngrößenanteile aus der Bodenprobe an. Die Bodenart wird nach dem
            KA5-Bodenartendiagramm bestimmt.
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-3 gap-x-3">
          {FRACTION_FIELDS.map(({ key, label }) => (
            <div key={key} className="flex flex-col gap-y-2">
              <Label htmlFor={`soil-fraction-${key}`}>{label} [%]</Label>
              <Input
                id={`soil-fraction-${key}`}
                type="number"
                inputMode="numeric"
                min={0}
                max={100}
                step={1}
                value={fractions[key]}
                onChange={handleChange(key)}
              />
            </div>
          ))}
        </div>
        <SoilTextureTriangle
          silt={fractions.silt}
          clay={fractions.clay}
          activeCondition={condition}
        />
        <p aria-live="polite" className="text-sm">
          Ermittelte Bodenart:{' '}
          <span className="font-semibold">{soilConditionLabel(condition)}</span>
        </p>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            Abbrechen
          </Button>
          <Button
            type="button"
            onClick={() => {
              onApply(condition)
              onOpenChange(false)
            }}
          >
            Übernehmen
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default SoilTextureDialog
