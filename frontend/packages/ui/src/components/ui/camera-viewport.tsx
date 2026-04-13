import * as React from 'react'

import { cn } from '@/lib/utils'

export type CameraViewportState = 'inactive' | 'loading' | 'scanning' | 'success' | 'error'

export interface CameraViewportProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Ref forwarded to the internal <video> element */
  videoRef: React.Ref<HTMLVideoElement>
  /** Visual state — drives overlay rendering and video visibility */
  state: CameraViewportState
  /** Content rendered centered over the dark backplate when state !== 'scanning' */
  overlay?: React.ReactNode
  /** When true, plays the one-shot scan-success flash animation */
  flash?: boolean
  /** Accessible label for the camera region */
  ariaLabel?: string
}

const Corner = ({ className, flash }: { className: string; flash: boolean }) => (
  <span
    aria-hidden="true"
    data-flash={flash ? 'true' : 'false'}
    className={cn(
      'absolute block size-7 md:size-9 border-white/90',
      'motion-safe:data-[flash=true]:animate-corner-pulse',
      className,
    )}
  />
)

const CameraViewport = React.forwardRef<HTMLDivElement, CameraViewportProps>(
  (
    { videoRef, state, overlay, flash = false, ariaLabel = 'Kamera-Vorschau', className, ...props },
    ref,
  ) => {
    const showVideo = state === 'scanning' || state === 'success'

    const [flashTick, setFlashTick] = React.useState(0)
    React.useEffect(() => {
      if (flash) setFlashTick((t) => t + 1)
    }, [flash])

    return (
      <div
        ref={ref}
        data-slot="camera-viewport"
        data-state={state}
        role="region"
        aria-label={ariaLabel}
        className={cn(
          'relative aspect-square w-full max-w-md mx-auto',
          'overflow-hidden rounded-2xl bg-dark',
          'shadow-[inset_0_0_0_1px_rgb(255_255_255/0.05)]',
          className,
        )}
        {...props}
      >
        {/* Layer 0: Video */}
        <video
          ref={videoRef}
          aria-hidden="true"
          muted
          playsInline
          autoPlay
          className={cn(
            'absolute inset-0 h-full w-full object-cover transition-opacity duration-300',
            showVideo ? 'opacity-100' : 'opacity-0',
          )}
        />

        {/* Layer 1: Spotlight ring + corner brackets — centered 70% square */}
        {(state === 'scanning' || state === 'success') && (
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 flex items-center justify-center"
          >
            <div className="relative size-[70%]">
              {/* Spotlight: huge dark ring around target area via box-shadow */}
              <div
                className={cn(
                  'absolute inset-0 rounded-md',
                  'shadow-[0_0_0_9999px_rgb(0_0_0/0.35)]',
                )}
              />
              {/* Corner brackets */}
              <Corner
                key={`tl-${flashTick}`}
                flash={flash}
                className="top-0 left-0 border-t-2 border-l-2 rounded-tl-md"
              />
              <Corner
                key={`tr-${flashTick}`}
                flash={flash}
                className="top-0 right-0 border-t-2 border-r-2 rounded-tr-md"
              />
              <Corner
                key={`bl-${flashTick}`}
                flash={flash}
                className="bottom-0 left-0 border-b-2 border-l-2 rounded-bl-md"
              />
              <Corner
                key={`br-${flashTick}`}
                flash={flash}
                className="bottom-0 right-0 border-b-2 border-r-2 rounded-br-md"
              />

              {/* Layer 2: Scan line — only while scanning */}
              {state === 'scanning' && (
                <div className="absolute inset-x-0 top-0 h-full overflow-hidden">
                  <div
                    aria-hidden="true"
                    className={cn(
                      'absolute inset-x-0 top-0 h-px w-full',
                      'bg-gradient-to-r from-transparent via-green-light-600 to-transparent',
                      'motion-safe:animate-scan-line motion-reduce:animate-scan-pulse',
                    )}
                  />
                </div>
              )}
            </div>
          </div>
        )}

        {/* Layer 3: Flash overlay on success */}
        {flash && (
          <div
            key={`flash-${flashTick}`}
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 animate-scan-flash"
          />
        )}

        {/* State overlay slot — loading spinner, permission icon, etc. */}
        {!showVideo && overlay && (
          <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 p-6 text-white/80">
            {overlay}
          </div>
        )}
      </div>
    )
  },
)
CameraViewport.displayName = 'CameraViewport'

export { CameraViewport }
