import BackLink from '@/components/general/links/BackLink'
import type { WizardStep } from './state'
import { Button, Stepper, type StepDefinition } from '@green-ecolution/ui'
import { ChevronLeft, ChevronRight } from 'lucide-react'

const STEPS: StepDefinition[] = [
  { id: 'scan', label: 'QR-Scan' },
  { id: 'gps', label: 'GPS' },
  { id: 'tree', label: 'Baum' },
  { id: 'confirm', label: 'Bestätigen' },
]

interface SensorWizardLayoutProps {
  step: WizardStep
  completedSteps: number[]
  onStepClick: (step: WizardStep) => void
  onBack?: () => void
  onNext?: () => void
  canGoNext: boolean
  nextLabel?: string
  hideFooter?: boolean
  children: React.ReactNode
}

const SensorWizardLayout = ({
  step,
  completedSteps,
  onStepClick,
  onBack,
  onNext,
  canGoNext,
  nextLabel = 'Weiter',
  hideFooter = false,
  children,
}: SensorWizardLayoutProps) => {
  return (
    <div className="container mt-6 pb-[env(safe-area-inset-bottom)]">
      <BackLink label="Zurück zur Sensorübersicht" link={{ to: '/sensors' }} />

      <article className="2xl:w-4/5 mb-6 md:mb-8">
        <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
          Sensor hinzufügen
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Geführter Ablauf zum Erfassen und Zuordnen eines Sensors.
        </p>
      </article>

      <Stepper
        steps={STEPS}
        currentStep={step}
        completedSteps={completedSteps}
        onStepClick={(target) => onStepClick(target as WizardStep)}
        className="mb-6 md:mb-8"
      />

      <div className="mx-auto w-full max-w-3xl">{children}</div>

      {!hideFooter && (onBack ?? onNext) && (
        <div className="mx-auto mt-8 flex w-full max-w-3xl flex-col gap-2 sm:flex-row sm:justify-between">
          {onBack ? (
            <Button variant="outline" onClick={onBack} className="sm:w-auto">
              <ChevronLeft className="size-4" />
              Zurück
            </Button>
          ) : (
            <span />
          )}
          {onNext && (
            <Button onClick={onNext} disabled={!canGoNext} className="sm:w-auto">
              {nextLabel}
              <ChevronRight className="size-4" />
            </Button>
          )}
        </div>
      )}
    </div>
  )
}

export default SensorWizardLayout
