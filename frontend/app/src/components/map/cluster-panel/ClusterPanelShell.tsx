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
      className="absolute right-4 top-4 z-10 text-dark-400 hover:text-dark-600"
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
        className="absolute z-[1030] top-4 right-4 bottom-4 w-[26rem] max-w-[calc(100%-2rem)] overflow-y-auto rounded-xl bg-white p-6 shadow-xl font-nunito-sans"
      >
        {closeButton}
        {children}
      </div>
    )
  }

  return (
    <>
      <div
        onClick={onClose}
        className="fixed inset-0 z-[1020] bg-dark-900/80"
        aria-hidden="true"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Baumgruppen-Details"
        className="fixed z-[1030] inset-x-3 bottom-3 top-24 overflow-y-auto rounded-xl bg-white p-6 shadow-xl font-nunito-sans"
      >
        {closeButton}
        {children}
      </div>
    </>
  )
}

export default ClusterPanelShell
