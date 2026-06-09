import { ReactNode, useCallback, useEffect } from 'react'

interface ClusterPanelShellProps {
  onClose: () => void
  children: ReactNode
}

const ClusterPanelShell = ({ onClose, children }: ClusterPanelShellProps) => {
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

  return <div className="flex h-full flex-col overflow-y-auto p-6 font-nunito-sans">{children}</div>
}

export default ClusterPanelShell
