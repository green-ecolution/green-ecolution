import type { PropsWithChildren, ReactNode } from 'react'
import { useState } from 'react'
import { Button, cn, Drawer, DrawerContent, DrawerTitle } from '@green-ecolution/ui'
import { X } from 'lucide-react'
import { useMediaQuery } from '@/hooks/useMediaQuery'

interface MapPanelProps extends PropsWithChildren {
  title: string
  onClose: () => void
  className?: string
  // Accessible label for the close button (the panel context differs per flow).
  closeLabel?: string
  // Extra header control rendered left of the close button (e.g. an edit action).
  headerAction?: ReactNode
  // Collapsed height of the mobile bottom-sheet; tune per flow so longer forms
  // show a bit more before the user drags up.
  mobileCollapsedSnap?: string
  // Controlled mobile snap point. Pass both to let the caller expand the sheet
  // (e.g. on entering edit mode); omit to let MapPanel manage it internally.
  activeSnapPoint?: number | string | null
  setActiveSnapPoint?: (snap: number | string | null) => void
}

const MapPanel = ({
  title,
  onClose,
  className,
  closeLabel = 'Abbrechen',
  headerAction,
  children,
  mobileCollapsedSnap = '360px',
  activeSnapPoint,
  setActiveSnapPoint,
}: MapPanelProps) => {
  const isDesktop = useMediaQuery('(min-width: 1024px)')
  const [internalSnap, setInternalSnap] = useState<number | string | null>(mobileCollapsedSnap)
  const snapControlled = setActiveSnapPoint !== undefined
  const snapPoint = snapControlled ? (activeSnapPoint ?? null) : internalSnap
  const setSnapPoint = setActiveSnapPoint ?? setInternalSnap
  // Collapsed snap keeps the map visible/tappable so the user can pick trees or a
  // location while the panel stays reachable; dragging up to `1` reveals it fully.
  const snapPoints: (number | string)[] = [mobileCollapsedSnap, 1]

  const headerControls = (
    <div className="flex items-center gap-1">
      {headerAction}
      <Button variant="ghost" size="icon" aria-label={closeLabel} onClick={onClose}>
        <X />
      </Button>
    </div>
  )

  if (!isDesktop) {
    return (
      <Drawer
        open
        onOpenChange={(open) => {
          if (!open) onClose()
        }}
        modal={false}
        snapPoints={snapPoints}
        activeSnapPoint={snapPoint}
        setActiveSnapPoint={setSnapPoint}
      >
        <DrawerContent showOverlay={false}>
          <div className="flex shrink-0 items-center justify-between gap-4 px-5 pb-3 pt-1">
            <DrawerTitle className="font-lato text-lg font-semibold">{title}</DrawerTitle>
            {headerControls}
          </div>
          <div className="flex min-h-0 flex-1 flex-col overflow-y-auto px-5 pb-5">{children}</div>
        </DrawerContent>
      </Drawer>
    )
  }

  return (
    <div
      className={cn(
        'absolute top-4 right-4 z-[1030] flex max-h-[calc(100%-2rem)] w-[30rem] max-w-[calc(100%-2rem)] flex-col rounded-xl bg-white p-5 font-nunito-sans shadow-xl',
        className,
      )}
    >
      <div className="mb-4 flex shrink-0 items-center justify-between gap-4">
        <h2 className="font-lato text-lg font-semibold">{title}</h2>
        {headerControls}
      </div>
      {children}
    </div>
  )
}

export default MapPanel
