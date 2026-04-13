import QRScannerDebugView from '@/components/debug/QRScannerDebugView'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/debug/qr-scanner/')({
  component: QRScannerDebugPage,
  loader: () => ({
    crumb: {
      title: 'QR-Scanner',
    },
  }),
})

function QRScannerDebugPage() {
  return (
    <div className="container mt-6 pb-[env(safe-area-inset-bottom)]">
      <article className="2xl:w-4/5 mb-8">
        <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
          QR-Scanner Debug
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Technische Inspektion des QR-Scan-Verfahrens: Browser-Fähigkeiten, Berechtigungs-Status,
          kontinuierliche Erkennung mit Rohdaten-Protokoll. Ideal zum Testen der Hardware- und
          Detector-Seite unabhängig vom Produktiv-Flow.
        </p>
      </article>
      <QRScannerDebugView />
    </div>
  )
}
