import { ReactNode, useCallback, useEffect } from 'react'
import { X } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import useMapInteractions from '@/hooks/useMapInteractions'
import { useMediaQuery } from '@/hooks/useMediaQuery'

interface ClusterPanelShellProps {
  onClose: () => void
  children: ReactNode
}

const ClusterPanelShell = ({ onClose, children }: ClusterPanelShellProps) => {
  const { enableDragging, disableDragging } = useMapInteractions()
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')

  useEffect(() => {
    disableDragging()
    return () => enableDragging()
  }, [disableDragging, enableDragging])

  const handleEscape = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    },
    [onClose],
  )

  useEffect(() => {
    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [handleEscape])

  const closeButton = (
    <Button
      variant="ghost"
      size="icon"
      aria-label="Seitenansicht schließen"
      className="text-dark-400 hover:bg-dark-50 hover:text-dark-700"
      onClick={onClose}
    >
      <X />
    </Button>
  )

  if (isLargeScreen) {
    return (
      <div
        role="dialog"
        aria-label="Baumgruppen-Details"
        onMouseEnter={disableDragging}
        onMouseLeave={enableDragging}
        className="absolute right-4 top-4 bottom-4 z-[1030] flex w-[28rem] max-w-[calc(100%-2rem)] flex-col rounded-2xl border border-dark-100 bg-white font-nunito-sans shadow-xl animate-in fade-in slide-in-from-right-4 duration-200"
      >
        <div className="flex shrink-0 justify-end px-3 pt-3">{closeButton}</div>
        <div className="grow overflow-y-auto px-6 pb-7">{children}</div>
      </div>
    )
  }

  return (
    <>
      <div
        onClick={onClose}
        className="fixed inset-0 z-[1020] bg-dark-900/60 backdrop-blur-sm animate-in fade-in"
        aria-hidden="true"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Baumgruppen-Details"
        className="fixed inset-x-0 bottom-0 top-24 z-[1030] flex flex-col rounded-t-2xl bg-white font-nunito-sans shadow-xl animate-in slide-in-from-bottom-4 duration-200"
      >
        <div className="flex justify-center pt-3">
          <span className="h-1.5 w-12 rounded-full bg-dark-200" aria-hidden="true" />
        </div>
        <div className="flex shrink-0 justify-end px-3">{closeButton}</div>
        <div className="grow overflow-y-auto px-5 pb-7">{children}</div>
      </div>
    </>
  )
}

export default ClusterPanelShell
