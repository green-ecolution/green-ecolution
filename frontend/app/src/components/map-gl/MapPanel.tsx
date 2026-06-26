import type { PropsWithChildren } from 'react'
import { Button, cn } from '@green-ecolution/ui'
import { X } from 'lucide-react'

interface MapPanelProps extends PropsWithChildren {
  title: string
  onClose: () => void
  className?: string
}

const MapPanel = ({ title, onClose, className, children }: MapPanelProps) => (
  <div
    className={cn(
      'absolute top-4 right-4 z-[1030] flex max-h-[calc(100%-2rem)] w-[30rem] max-w-[calc(100%-2rem)] flex-col rounded-xl bg-white p-5 font-nunito-sans shadow-xl',
      className,
    )}
  >
    <div className="mb-4 flex shrink-0 items-center justify-between gap-4">
      <h2 className="font-lato text-lg font-semibold">{title}</h2>
      <Button variant="ghost" size="icon" aria-label="Abbrechen" onClick={onClose}>
        <X />
      </Button>
    </div>
    {children}
  </div>
)

export default MapPanel
