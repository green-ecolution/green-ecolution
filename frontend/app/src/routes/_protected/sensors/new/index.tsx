import { queryRoots } from '@/api/queries'
import { sensorApi } from '@/api/backendApi'
import { sensorQueries } from '@/api/queries'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'
import { mapActivateError, resolveResponseStatus } from '@/api/sensorErrors'
import SensorReviewStep from '@/components/sensor/wizard/SensorReviewStep'
import SensorScanStep from '@/components/sensor/wizard/SensorScanStep'
import SensorTargetStep from '@/components/sensor/wizard/SensorTargetStep'
import SensorWizardLayout from '@/components/sensor/wizard/SensorWizardLayout'
import SensorWizardSuccess from '@/components/sensor/wizard/SensorWizardSuccess'
import {
  INITIAL_WIZARD_STATE,
  normalizeSensorId,
  wizardReducer,
  type WizardStep,
} from '@/components/sensor/wizard/state'
import useGeolocation from '@/hooks/useGeolocation'
import { useMutation, useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useReducer } from 'react'
import { useTranslation } from 'react-i18next'
import { sensorTargetKinds, targetDescriptor, type SensorTargetKind } from '@/lib/targets'
import type { SelectedTarget } from '@/modules/types'
import { useActiveModules } from '@/modules/useActiveModules'

export const Route = createFileRoute('/_protected/sensors/new/')({
  component: NewSensor,
})

function NewSensor() {
  const navigate = useNavigate()
  const { i18n } = useTranslation()
  const invalidate = useInvalidateQueryRoots()
  const { manifests } = useActiveModules()
  const [state, dispatch] = useReducer(wizardReducer, INITIAL_WIZARD_STATE)
  const kinds = sensorTargetKinds(manifests)
  const kind =
    kinds.length === 1
      ? kinds[0]
      : kinds.find(
          ({ module, target }) =>
            module === state.targetKind?.module && target.kind === state.targetKind.kind,
        )
  const {
    status: gpsStatus,
    position,
    errorMessage: gpsError,
    stop,
    relocate,
  } = useGeolocation({
    autoStart: true,
  })

  // Freeze the live fix once a sensor was scanned; later steps work on the snapshot.
  useEffect(() => {
    if (state.sensorId && !state.frozenFix && position) {
      dispatch({ type: 'gpsFrozen', fix: position })
      stop()
    }
  }, [state.sensorId, state.frozenFix, position, stop])

  const sensorLookup = useQuery({
    ...sensorQueries.detail(state.sensorId ?? ''),
    enabled: !!state.sensorId,
    retry: false,
  })

  const verifiedSensor = sensorLookup.data?.status === 'prepared' ? sensorLookup.data : null

  const completedSteps = getCompletedSteps({
    sensorVerified: Boolean(verifiedSensor),
    targetSelected: Boolean(state.selectedTarget),
  })

  const activateMutation = useMutation({
    mutationFn: ({ sensorId, selected }: { sensorId: string; selected: SelectedTarget }) =>
      sensorApi.activateSensor({ sensorId, activateSensorRequest: { target: selected.target } }),
    onMutate: () => dispatch({ type: 'submissionStart' }),
    onSuccess: async (_, { selected }) => {
      dispatch({ type: 'submissionSuccess' })
      const moduleRoots = targetDescriptor(manifests, selected.target)?.invalidate ?? []
      await invalidate([...queryRoots.sensor, ...moduleRoots])
    },
    onError: (err, { selected }) =>
      dispatch({
        type: 'submissionError',
        message: mapActivateError(err, targetDescriptor(manifests, selected.target)),
      }),
  })

  const handleRelocate = useCallback(async () => {
    dispatch({ type: 'gpsCleared' })
    const next = await relocate().catch(() => null)
    if (next) {
      dispatch({ type: 'gpsFrozen', fix: next })
      stop()
    }
  }, [relocate, stop])

  const handleSelectTarget = useCallback(
    (selected: SelectedTarget) => dispatch({ type: 'targetSelected', selected }),
    [],
  )

  const handleChooseKind = useCallback(
    (chosen: SensorTargetKind | undefined) =>
      dispatch({
        type: 'targetKindChosen',
        kind: chosen && { module: chosen.module, kind: chosen.target.kind },
      }),
    [],
  )

  const targetStepLabel = kinds.length === 1 ? i18n.t(kinds[0].target.labelKey) : undefined

  const handleStepClick = useCallback(
    (target: WizardStep) => dispatch({ type: 'goToStep', step: target }),
    [],
  )

  const handleBack = useCallback(() => {
    if (state.step > 1) {
      dispatch({ type: 'goToStep', step: (state.step - 1) as WizardStep })
    }
  }, [state.step])

  const handleNext = useCallback(() => {
    if (state.step < 3) {
      dispatch({ type: 'goToStep', step: (state.step + 1) as WizardStep })
    }
  }, [state.step])

  const handleResetForNext = useCallback(() => {
    dispatch({ type: 'resetForNextSensor' })
    void relocate()
  }, [relocate])

  const handleBackToOverview = useCallback(() => {
    void navigate({ to: '/sensors', search: { page: 1 } })
  }, [navigate])

  if (state.submission === 'success') {
    return (
      <SensorWizardLayout
        step={3}
        targetStepLabel={targetStepLabel}
        completedSteps={[1, 2, 3]}
        onStepClick={handleStepClick}
        canGoNext={false}
        hideFooter
      >
        <SensorWizardSuccess
          sensorId={state.sensorId ?? ''}
          targetLabel={state.selectedTarget?.label ?? ''}
          onNext={handleResetForNext}
          onBackToOverview={handleBackToOverview}
        />
      </SensorWizardLayout>
    )
  }

  const { sensorId, selectedTarget } = state
  const canGoNext =
    (state.step === 1 && Boolean(verifiedSensor)) ||
    (state.step === 2 && Boolean(state.selectedTarget))

  return (
    <SensorWizardLayout
      step={state.step}
      targetStepLabel={targetStepLabel}
      completedSteps={completedSteps}
      onStepClick={handleStepClick}
      onBack={state.step === 1 ? undefined : handleBack}
      onNext={state.step === 3 || state.step === 1 ? undefined : handleNext}
      canGoNext={canGoNext}
    >
      {state.step === 1 && (
        <SensorScanStep
          scannedSensorId={state.sensorId}
          isLookupLoading={!!state.sensorId && sensorLookup.isFetching}
          isLookupError={sensorLookup.isError}
          lookupErrorStatus={resolveResponseStatus(sensorLookup.error)}
          sensor={sensorLookup.data ?? null}
          onScanned={(id) => {
            dispatch({ type: 'qrScanned', sensorId: normalizeSensorId(id) })
            if (position) {
              dispatch({ type: 'gpsFrozen', fix: position })
              stop()
            }
          }}
          onScanAgain={() => dispatch({ type: 'scanCleared' })}
          onRetryLookup={() => void sensorLookup.refetch()}
          onContinue={handleNext}
        />
      )}
      {state.step === 2 && (
        <SensorTargetStep
          kinds={kinds}
          kind={kind}
          onChooseKind={handleChooseKind}
          current={state.selectedTarget?.target}
          onSelect={handleSelectTarget}
          position={state.frozenFix}
          geolocationStatus={gpsStatus}
          geolocationError={gpsError}
          onRelocate={() => void handleRelocate()}
        />
      )}
      {state.step === 3 && sensorId && selectedTarget && (
        <SensorReviewStep
          sensorId={sensorId}
          target={selectedTarget}
          position={state.frozenFix}
          status={state.submission}
          errorMessage={state.errorMessage}
          onActivate={() => activateMutation.mutate({ sensorId, selected: selectedTarget })}
        />
      )}
    </SensorWizardLayout>
  )
}

function getCompletedSteps(args: { sensorVerified: boolean; targetSelected: boolean }) {
  const done: number[] = []
  if (!args.sensorVerified) return done
  done.push(1)
  if (!args.targetSelected) return done
  done.push(2)
  return done
}
