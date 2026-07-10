import type { ComponentProps } from 'react'
import { Button, cn } from '@green-ecolution/ui'

type MapControlButtonProps = ComponentProps<typeof Button> & { active?: boolean }

const MapControlButton = ({ active = false, className, ...props }: MapControlButtonProps) => (
  <Button
    variant="outline"
    size="icon"
    className={cn(
      'rounded-full border-0 shadow-cards',
      active
        ? 'bg-dark-800 text-white hover:bg-dark-800/90 hover:text-white'
        : 'bg-white text-dark-800',
      className,
    )}
    {...props}
  />
)

export default MapControlButton
