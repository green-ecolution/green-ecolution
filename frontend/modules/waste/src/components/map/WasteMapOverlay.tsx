import { Suspense, useCallback } from 'react'
import { useLocation, useNavigate } from '@tanstack/react-router'
import { useWasteMapSearch } from '../../lib/mapSearch'
import WasteBinPopup from './WasteBinPopup'

const WasteMapOverlay = () => {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { wasteBin } = useWasteMapSearch()
  const isIndex = pathname === '/map' || pathname === '/map/'

  const close = useCallback(() => {
    navigate({ to: '/map', search: (prev) => ({ ...prev, wasteBin: undefined }) }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }, [navigate])

  if (!isIndex || !wasteBin) return null
  return (
    <Suspense fallback={null}>
      <WasteBinPopup binId={wasteBin} onClose={close} />
    </Suspense>
  )
}

export default WasteMapOverlay
