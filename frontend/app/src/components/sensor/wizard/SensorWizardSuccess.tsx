import { Button } from '@green-ecolution/ui'
import { CheckCircle2, RotateCw, ListChecks } from 'lucide-react'

interface SensorWizardSuccessProps {
  sensorId: string
  treeNumber: string
  onNext: () => void
  onBackToOverview: () => void
}

const SensorWizardSuccess = ({
  sensorId,
  treeNumber,
  onNext,
  onBackToOverview,
}: SensorWizardSuccessProps) => {
  return (
    <div className="mx-auto max-w-2xl rounded-2xl border border-green-dark/30 bg-green-dark-50/30 p-6 md:p-8">
      <div className="flex flex-col items-center text-center space-y-4">
        <div className="rounded-full bg-green-dark/10 p-3">
          <CheckCircle2 className="size-8 text-green-dark" aria-hidden />
        </div>
        <h2 className="text-2xl font-semibold">Sensor aktiviert</h2>
        <p className="text-sm text-muted-foreground max-w-prose">
          Sensor <span className="font-mono">{sensorId}</span> wurde Baum{' '}
          <span className="font-mono">{treeNumber}</span> zugeordnet.
        </p>

        <div className="grid w-full grid-cols-1 gap-2 sm:grid-cols-2 mt-2">
          <Button onClick={onNext}>
            <RotateCw className="size-4" />
            Nächsten Sensor scannen
          </Button>
          <Button variant="outline" onClick={onBackToOverview}>
            <ListChecks className="size-4" />
            Zur Übersicht
          </Button>
        </div>
      </div>
    </div>
  )
}

export default SensorWizardSuccess
