import createToast from '@/hooks/createToast'
import { Button, Card, CardContent, CardFooter, CardHeader, CardTitle } from '@green-ecolution/ui'
import { ArrowRight, CheckCircle2, RotateCcw } from 'lucide-react'

interface QRScanResultProps {
  sensorId: string
  onScanAgain: () => void
}

const QRScanResult = ({ sensorId, onScanAgain }: QRScanResultProps) => {
  const showToast = createToast()

  const handleContinue = () => {
    showToast('Nächster Schritt ist noch nicht implementiert', 'success')
  }

  return (
    <Card variant="outlined" className="animate-in fade-in-0 slide-in-from-bottom-2 duration-300">
      <CardHeader>
        <div className="flex items-center gap-2">
          <CheckCircle2 aria-hidden="true" className="size-5 text-green-dark" />
          <CardTitle className="text-xl">QR-Code erkannt</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        <span className="text-xs uppercase tracking-widest text-muted-foreground">Sensor-ID</span>
        <code className="font-mono text-lg md:text-xl font-semibold break-all bg-dark-50 rounded-lg px-3 py-2 border border-dark-100">
          {sensorId}
        </code>
      </CardContent>
      <CardFooter className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2">
        <Button variant="outline" size="sm" onClick={onScanAgain} className="w-full sm:w-auto">
          <RotateCcw />
          Erneut scannen
        </Button>
        <Button
          variant="default"
          size="sm"
          onClick={handleContinue}
          autoFocus
          className="w-full sm:w-auto"
        >
          Weiter
          <ArrowRight />
        </Button>
      </CardFooter>
    </Card>
  )
}

export default QRScanResult
