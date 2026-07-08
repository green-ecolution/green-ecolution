import useStore from '@/store/store'
import { useMediaQuery } from './useMediaQuery'

export function useSidebarCollapsed(): boolean {
  const userChoice = useStore((s) => s.sidebarCollapsed)
  const isDesktop = useMediaQuery('(min-width: 1280px)')
  return userChoice ?? !isDesktop
}
