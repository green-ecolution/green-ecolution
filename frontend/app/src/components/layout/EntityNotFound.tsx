import { Link, LinkProps } from '@tanstack/react-router'
import { MoveLeft, Search } from 'lucide-react'
import { Button } from '@green-ecolution/ui'

interface EntityNotFoundProps {
  entityName: string
  backTo: LinkProps['to']
  backLabel: string
}

function EntityNotFound({ entityName, backTo, backLabel }: EntityNotFoundProps) {
  return (
    <div className="container mt-6">
      <section className="relative my-12 lg:my-20">
        <div className="flex flex-col items-center text-center animate-[fadeInUp_0.6s_ease-out]">
          {/* Icon */}
          <div className="relative mb-8">
            <div className="absolute inset-0 bg-green-light-200/40 dark:bg-green-dark-800/40 rounded-full blur-2xl scale-150" />
            <div className="relative w-20 h-20 lg:w-24 lg:h-24 flex items-center justify-center rounded-full bg-gradient-to-br from-green-light-100 to-green-dark-100 dark:from-green-dark-800 dark:to-green-dark-900 border border-green-dark-200/30 dark:border-green-dark-700/50">
              <Search
                className="w-8 h-8 lg:w-10 lg:h-10 text-green-dark-500 dark:text-green-dark-400"
                strokeWidth={1.5}
              />
            </div>
          </div>

          {/* Content */}
          <div className="max-w-md space-y-3">
            <h1 className="font-lato font-bold text-2xl lg:text-3xl text-dark dark:text-light">
              {entityName} nicht gefunden
            </h1>
            <p className="text-dark-500 dark:text-dark-400 leading-relaxed">
              Die angeforderte Ressource existiert nicht oder wurde bereits entfernt.
            </p>
          </div>

          {/* Action button */}
          <div className="mt-8">
            <Button asChild variant="outline" className="group gap-2 px-6">
              <Link to={backTo}>
                <MoveLeft className="w-4 h-4 transition-transform group-hover:-translate-x-1" />
                {backLabel}
              </Link>
            </Button>
          </div>
        </div>
      </section>
    </div>
  )
}

export default EntityNotFound
