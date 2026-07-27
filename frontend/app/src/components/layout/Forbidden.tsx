import { Link } from '@tanstack/react-router'
import { Lock, MoveRight } from 'lucide-react'
import { Button } from '@green-ecolution/ui'

function Forbidden() {
  return (
    <div className="container mt-6">
      <section className="relative my-12 lg:my-20">
        <div className="flex flex-col items-center text-center animate-[fadeInUp_0.6s_ease-out] motion-reduce:animate-none">
          {/* Concentric ring: something is enclosed and the viewer stands outside.
              Deliberately neutral, not green — green marks healthy state elsewhere. */}
          <div className="mb-8 flex h-28 w-28 items-center justify-center rounded-full border border-dark-200/60 dark:border-dark-600/60 lg:h-32 lg:w-32">
            <div className="flex h-20 w-20 items-center justify-center rounded-full bg-dark-100 dark:bg-dark-800 lg:h-24 lg:w-24">
              <Lock
                className="h-8 w-8 text-dark-500 dark:text-dark-300 lg:h-10 lg:w-10"
                strokeWidth={1.5}
              />
            </div>
          </div>

          <div className="max-w-md space-y-3">
            <h1 className="font-lato font-bold text-2xl lg:text-3xl text-dark dark:text-light">
              Kein Zugriff
            </h1>
            <p className="text-dark-500 dark:text-dark-400 leading-relaxed">
              Für diesen Bereich fehlt dir die nötige Berechtigung. Wende dich an die Administration
              deiner Organisation, wenn du Zugriff benötigst.
            </p>
          </div>

          <div className="mt-8">
            <Button asChild variant="outline" className="group gap-2 px-6">
              <Link to="/dashboard">
                Zum Dashboard
                <MoveRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
              </Link>
            </Button>
          </div>
        </div>
      </section>
    </div>
  )
}

export default Forbidden
