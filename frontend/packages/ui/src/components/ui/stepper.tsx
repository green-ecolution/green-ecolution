import * as React from 'react'
import { Check } from 'lucide-react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const circleVariants = cva(
  'flex size-8 shrink-0 items-center justify-center rounded-full border text-sm font-semibold tabular-nums transition-colors',
  {
    variants: {
      state: {
        upcoming: 'border-dark-200 bg-background text-dark-500',
        current: 'border-green-dark bg-green-dark text-white ring-4 ring-green-dark/15',
        completed: 'border-green-dark bg-green-dark text-white',
      },
    },
    defaultVariants: {
      state: 'upcoming',
    },
  },
)

type CircleState = NonNullable<VariantProps<typeof circleVariants>['state']>

export type StepDefinition = {
  id: string
  label: string
}

export type StepperProps = {
  steps: StepDefinition[]
  currentStep: number
  completedSteps: number[]
  onStepClick?: (step: number) => void
  className?: string
}

const stateFor = (
  index1: number,
  currentStep: number,
  completed: ReadonlySet<number>,
): CircleState => {
  if (completed.has(index1)) return 'completed'
  if (index1 === currentStep) return 'current'
  return 'upcoming'
}

export const Stepper = ({
  steps,
  currentStep,
  completedSteps,
  onStepClick,
  className,
}: StepperProps) => {
  const completed = React.useMemo(() => new Set(completedSteps), [completedSteps])

  return (
    <nav aria-label="Fortschritt">
      <p className="md:hidden mb-2 text-xs text-muted-foreground">
        Schritt {currentStep} von {steps.length}
      </p>
      <ol className={cn('flex items-center gap-2 md:gap-4', className)}>
        {steps.map((step, idx) => {
          const index1 = idx + 1
          const state = stateFor(index1, currentStep, completed)
          const isClickable = state === 'completed' && Boolean(onStepClick)
          const isLast = idx === steps.length - 1

          return (
            <li
              key={step.id}
              className="flex flex-1 items-center gap-2 md:gap-3"
              aria-current={state === 'current' ? 'step' : undefined}
              aria-disabled={state === 'upcoming' ? true : undefined}
            >
              {isClickable ? (
                <button
                  type="button"
                  onClick={() => onStepClick?.(index1)}
                  className="flex items-center gap-2 md:gap-3 rounded-md cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-dark/40"
                >
                  <span className={circleVariants({ state })}>
                    <Check className="size-4" aria-hidden />
                  </span>
                  <span className="hidden md:inline text-sm font-medium text-foreground">
                    {step.label}
                  </span>
                </button>
              ) : (
                <span className="flex items-center gap-2 md:gap-3">
                  <span className={circleVariants({ state })}>
                    {state === 'completed' ? <Check className="size-4" aria-hidden /> : index1}
                  </span>
                  <span
                    className={cn(
                      'hidden md:inline text-sm',
                      state === 'current'
                        ? 'font-semibold text-foreground'
                        : 'text-muted-foreground',
                    )}
                  >
                    {step.label}
                  </span>
                </span>
              )}
              {!isLast && (
                <span
                  aria-hidden
                  className={cn(
                    'h-px flex-1',
                    completed.has(index1) ? 'bg-green-dark' : 'bg-dark-200',
                  )}
                />
              )}
            </li>
          )
        })}
      </ol>
    </nav>
  )
}
