import { sensorApi } from '@/api/backendApi'
import { sensorIdQuery } from '@/api/queries'
import SensorGpsStep from '@/components/sensor/wizard/SensorGpsStep'
import SensorReviewStep from '@/components/sensor/wizard/SensorReviewStep'
import SensorScanStep from '@/components/sensor/wizard/SensorScanStep'
import SensorTreeStep from '@/components/sensor/wizard/SensorTreeStep'
import SensorWizardLayout from '@/components/sensor/wizard/SensorWizardLayout'
import SensorWizardSuccess from '@/components/sensor/wizard/SensorWizardSuccess'
import {
  INITIAL_WIZARD_STATE,
  normalizeSensorId,
  wizardReducer,
  type WizardStep,
} from '@/components/sensor/wizard/state'
import useGeolocation from '@/hooks/useGeolocation'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useReducer } from 'react'

export const Route = createFileRoute('/_protected/sensors/new/')({
  component: NewSensor,
})

const resolveResponseStatus = (err: unknown): number | null => {
  if (err instanceof Response) return err.status
  if (
    err != null &&
    typeof err === 'object' &&
    'response' in err &&
    err.response instanceof Response
  )
    return err.response.status
  return null
}

const mapActivateError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return 'Sensor existiert nicht (mehr). Bitte erneut scannen.'
  if (status === 409) return 'Sensor ist bereits einem Baum zugeordnet.'
  return 'Aktivierung fehlgeschlagen. Bitte erneut versuchen.'
}

function NewSensor() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [state, dispatch] = useReducer(wizardReducer, INITIAL_WIZARD_STATE)
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
    ...sensorIdQuery(state.sensorId ?? ''),
    enabled: !!state.sensorId,
    retry: false,
  })

  const verifiedSensor = sensorLookup.data?.status === 'prepared' ? sensorLookup.data : null

  const completedSteps = getCompletedSteps({
    sensorVerified: Boolean(verifiedSensor),
    frozenFix: state.frozenFix,
    selectedTreeId: state.selectedTreeId,
  })

  const activateMutation = useMutation({
    mutationFn: () =>
      sensorApi.activateSensor({
        sensorId: state.sensorId!,
        activateSensorRequest: { treeId: state.selectedTreeId! },
      }),
    onMutate: () => dispatch({ type: 'submissionStart' }),
    onSuccess: async () => {
      dispatch({ type: 'submissionSuccess' })
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['sensors'] }),
        queryClient.invalidateQueries({ queryKey: ['sensor', state.sensorId] }),
        queryClient.invalidateQueries({ queryKey: ['tree', state.selectedTreeId] }),
      ])
    },
    onError: (err) => dispatch({ type: 'submissionError', message: mapActivateError(err) }),
  })

  const handleRelocate = useCallback(async () => {
    dispatch({ type: 'gpsCleared' })
    const next = await relocate().catch(() => null)
    if (next) {
      dispatch({ type: 'gpsFrozen', fix: next })
      stop()
    }
  }, [relocate, stop])

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
    if (state.step < 4) {
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
        step={4}
        completedSteps={[1, 2, 3, 4]}
        onStepClick={handleStepClick}
        canGoNext={false}
        hideFooter
      >
        <SensorWizardSuccess
          sensorId={state.sensorId ?? ''}
          treeNumber={state.selectedTreeNumber ?? ''}
          onNext={handleResetForNext}
          onBackToOverview={handleBackToOverview}
        />
      </SensorWizardLayout>
    )
  }

  const canGoNext =
    (state.step === 1 && Boolean(verifiedSensor)) ||
    (state.step === 2 && Boolean(state.frozenFix)) ||
    (state.step === 3 && Boolean(state.selectedTreeId))

  return (
    <SensorWizardLayout
      step={state.step}
      completedSteps={completedSteps}
      onStepClick={handleStepClick}
      onBack={state.step === 1 ? undefined : handleBack}
      onNext={state.step === 4 || state.step === 1 ? undefined : handleNext}
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
        <SensorGpsStep
          position={state.frozenFix}
          status={gpsStatus}
          errorMessage={gpsError}
          onRelocate={() => void handleRelocate()}
        />
      )}
      {state.step === 3 && state.frozenFix && (
        <SensorTreeStep
          position={state.frozenFix}
          selectedTreeId={state.selectedTreeId}
          onSelect={(treeId, number, species) =>
            dispatch({ type: 'treeSelected', treeId, number, species })
          }
        />
      )}
      {state.step === 4 && state.frozenFix && state.sensorId && state.selectedTreeId && (
        <SensorReviewStep
          sensorId={state.sensorId}
          treeNumber={state.selectedTreeNumber ?? ''}
          treeSpecies={state.selectedTreeSpecies ?? ''}
          position={state.frozenFix}
          status={state.submission}
          errorMessage={state.errorMessage}
          onActivate={() => activateMutation.mutate()}
        />
      )}
    </SensorWizardLayout>
  )
}

function getCompletedSteps(args: {
  sensorVerified: boolean
  frozenFix: unknown
  selectedTreeId: string | null
}) {
  const done: number[] = []
  if (!args.sensorVerified) return done
  done.push(1)
  if (!args.frozenFix) return done
  done.push(2)
  if (!args.selectedTreeId) return done
  done.push(3)
  return done
}
