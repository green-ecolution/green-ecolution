import BackLink from '@/components/general/links/BackLink'
import type { WizardStep } from './state'
import { Button, Stepper, type StepDefinition } from '@green-ecolution/ui'
import { ChevronLeft, ChevronRight } from 'lucide-react'

const STEPS: StepDefinition[] = [
  { id: 'scan', label: 'QR-Scan' },
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

      <Stepper
        steps={STEPS}
        currentStep={step}
        completedSteps={completedSteps}
        onStepClick={(target) => onStepClick(target as WizardStep)}
        className="mt-6 mb-8 md:mt-8 md:mb-10"
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
