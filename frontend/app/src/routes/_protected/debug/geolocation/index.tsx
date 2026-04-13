import GeolocationDebugView from '@/components/debug/GeolocationDebugView'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/debug/geolocation/')({
  component: GeolocationDebugPage,
  loader: () => ({
    crumb: {
      title: 'GPS-Ortung',
    },
  }),
})

function GeolocationDebugPage() {
  return (
    <div className="container mt-6 pb-[env(safe-area-inset-bottom)]">
      <article className="2xl:w-4/5 mb-8">
        <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
          GPS-Ortung Debug
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Diagnose der Browser-Geolocation-API: Sicherheitskontext, Berechtigungs-Status,
          Live-Position mit Genauigkeitskreis und Verlaufsprotokoll. Ideal zum Testen der
          Hardware-Seite unabhängig vom Produktiv-Flow.
        </p>
      </article>
      <GeolocationDebugView />
    </div>
  )
}
