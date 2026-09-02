import { useTranslation } from 'react-i18next'
import BackLink from '@/components/general/links/BackLink'
import type { WizardStep } from './state'
import { Button, Stepper, type StepDefinition } from '@green-ecolution/ui'
import { ChevronLeft, ChevronRight } from 'lucide-react'

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
  nextLabel,
  hideFooter = false,
  children,
}: SensorWizardLayoutProps) => {
  const { t } = useTranslation(['sensor', 'common'])
  const steps: StepDefinition[] = [
    { id: 'scan', label: t('wizard.layout.steps.scan') },
    { id: 'tree', label: t('wizard.layout.steps.tree') },
    { id: 'confirm', label: t('common:actions.confirm') },
  ]

  return (
    <div className="container mt-6 pb-[env(safe-area-inset-bottom)]">
      <BackLink label={t('wizard.layout.backToList')} link={{ to: '/sensors' }} />

      <Stepper
        steps={steps}
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
              {t('common:actions.back')}
            </Button>
          ) : (
            <span />
          )}
          {onNext && (
            <Button onClick={onNext} disabled={!canGoNext} className="sm:w-auto">
              {nextLabel ?? t('common:actions.next')}
              <ChevronRight className="size-4" />
            </Button>
          )}
        </div>
      )}
    </div>
  )
}

export default SensorWizardLayout
