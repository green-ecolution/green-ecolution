import * as React from 'react'
import * as SliderPrimitive from '@radix-ui/react-slider'

import { cn } from '@/lib/utils'

export interface SliderProps extends React.ComponentPropsWithoutRef<typeof SliderPrimitive.Root> {
  showLabels?: boolean
}

const Slider = React.forwardRef<React.ComponentRef<typeof SliderPrimitive.Root>, SliderProps>(
  ({ className, showLabels, value, defaultValue, min = 0, max = 100, disabled, ...props }, ref) => {
    const currentValue = value ?? defaultValue ?? [min]
    const isRange = currentValue.length === 2
    const [isDragging, setIsDragging] = React.useState(false)

    React.useEffect(() => {
      if (!isDragging) return

      const handlePointerUp = () => setIsDragging(false)
      document.addEventListener('pointerup', handlePointerUp)
      return () => document.removeEventListener('pointerup', handlePointerUp)
    }, [isDragging])

    return (
      <div className="w-full">
        <SliderPrimitive.Root
          ref={ref}
          data-slot="slider"
          min={min}
          max={max}
          value={value}
          defaultValue={defaultValue}
          disabled={disabled}
          className={cn(
            'relative flex w-full touch-none select-none items-center',
            disabled && 'cursor-not-allowed opacity-50',
            className,
          )}
          {...props}
        >
          <SliderPrimitive.Track
            data-slot="slider-track"
            className={cn(
              'relative h-2 w-full grow overflow-hidden rounded-full bg-dark-200',
              disabled ? 'cursor-not-allowed' : 'cursor-pointer',
            )}
          >
            <SliderPrimitive.Range
              data-slot="slider-range"
              className="absolute h-full bg-green-dark"
            />
          </SliderPrimitive.Track>
          {currentValue.map((_, index) => (
            <SliderPrimitive.Thumb
              key={index}
              data-slot="slider-thumb"
              onPointerDown={() => !disabled && setIsDragging(true)}
              className={cn(
                'block h-5 w-5 rounded-full border-2 border-green-dark bg-white transition-[color,box-shadow] outline-none focus-visible:ring-[3px] focus-visible:ring-green-dark/50',
                disabled ? 'cursor-not-allowed' : isDragging ? 'cursor-grabbing' : 'cursor-grab',
              )}
            />
          ))}
        </SliderPrimitive.Root>
        {showLabels && (
          <div className="mt-2 flex justify-between text-sm text-dark-600">
            {isRange ? (
              <>
                <span>{currentValue[0]}</span>
                <span>{currentValue[1]}</span>
              </>
            ) : (
              <>
                <span>{min}</span>
                <span>{currentValue[0]}</span>
              </>
            )}
          </div>
        )}
      </div>
    )
  },
)
Slider.displayName = SliderPrimitive.Root.displayName

export { Slider }
