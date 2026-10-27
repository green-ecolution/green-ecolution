import { Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import { Button, InlineAlert, Loading } from '@green-ecolution/ui'
import { ChevronLeft } from 'lucide-react'
import type { SensorTargetKind } from '@/lib/targets'
import type { TargetWizardStepProps } from '@/modules/types'
import TargetKindChoice from '../TargetKindChoice'

interface SensorTargetStepProps extends TargetWizardStepProps {
  kinds: SensorTargetKind[]
  kind: SensorTargetKind | undefined
  onChooseKind: (kind: SensorTargetKind | undefined) => void
}

const SensorTargetStep = ({ kinds, kind, onChooseKind, ...stepProps }: SensorTargetStepProps) => {
  const { t } = useTranslation('sensor')

  if (kinds.length === 0) {
    return <InlineAlert variant="warning" description={t('wizard.target.noKinds')} />
  }

  if (!kind) {
    return (
      <div className="space-y-6">
        <header className="space-y-2">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl">
            {t('wizard.target.chooseKind.title')}
          </h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            {t('wizard.target.chooseKind.description')}
          </p>
        </header>
        <TargetKindChoice kinds={kinds} onChoose={onChooseKind} />
      </div>
    )
  }

  const { WizardStep } = kind.target
  return (
    <div className="space-y-4">
      {kinds.length > 1 && (
        <Button
          variant="link"
          size="sm"
          className="h-auto px-0"
          onClick={() => onChooseKind(undefined)}
        >
          <ChevronLeft className="size-4" />
          {t('wizard.target.changeKind')}
        </Button>
      )}
      <Suspense fallback={<Loading className="justify-center" />}>
        <WizardStep {...stepProps} />
      </Suspense>
    </div>
  )
}

export default SensorTargetStep
