import { ChangeEvent, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
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
import { useSoilConditionLabel } from '@/hooks/details/useDetailsForSoilCondition'
import {
  SoilFractions,
  balanceFractions,
  classifySoilTexture,
  regionMidpoint,
} from '@/lib/soilTexture'
import SoilTextureTriangle from './SoilTextureTriangle'

const NEUTRAL_FRACTIONS: SoilFractions = { sand: 33, silt: 34, clay: 33 }

interface SoilTextureFormProps {
  initialCondition: SoilCondition
  onApply: (condition: SoilCondition) => void
  onCancel: () => void
}

const SoilTextureForm = ({ initialCondition, onApply, onCancel }: SoilTextureFormProps) => {
  const { t } = useTranslation('treecluster')
  const soilConditionLabel = useSoilConditionLabel()
  const FRACTION_FIELDS: { key: keyof SoilFractions; label: string }[] = [
    { key: 'sand', label: t('soilTexture.fractionSand') },
    { key: 'silt', label: t('soilTexture.fractionSilt') },
    { key: 'clay', label: t('soilTexture.fractionClay') },
  ]
  const [fractions, setFractions] = useState<SoilFractions>(
    () => regionMidpoint(initialCondition) ?? NEUTRAL_FRACTIONS,
  )

  const condition = classifySoilTexture(fractions.silt, fractions.clay)

  const lastChanged = useRef<keyof SoilFractions | null>(null)

  const handleChange = (field: keyof SoilFractions) => (event: ChangeEvent<HTMLInputElement>) => {
    const previous = lastChanged.current
    const hold =
      previous && previous !== field
        ? previous
        : FRACTION_FIELDS.find(({ key }) => key !== field)!.key
    lastChanged.current = field
    setFractions((current) => balanceFractions(current, field, event.target.valueAsNumber, hold))
  }

  return (
    <>
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
        {t('soilTexture.resultLabel')}{' '}
        <span className="font-semibold">{soilConditionLabel(condition)}</span>
      </p>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          {t('soilTexture.cancelButton')}
        </Button>
        <Button type="button" onClick={() => onApply(condition)}>
          {t('soilTexture.applyButton')}
        </Button>
      </DialogFooter>
    </>
  )
}

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
  const { t } = useTranslation('treecluster')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('soilTexture.dialogTitle')}</DialogTitle>
          <DialogDescription>{t('soilTexture.dialogDescription')}</DialogDescription>
        </DialogHeader>
        <SoilTextureForm
          initialCondition={initialCondition}
          onApply={(condition) => {
            onApply(condition)
            onOpenChange(false)
          }}
          onCancel={() => onOpenChange(false)}
        />
      </DialogContent>
    </Dialog>
  )
}

export default SoilTextureDialog
