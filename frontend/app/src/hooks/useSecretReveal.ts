import { useCallback, useEffect, useState } from 'react'

/** Reveal toggle for sensitive values that auto-hides after `autoHideSeconds`. */
export const useSecretReveal = (autoHideSeconds = 10) => {
  const [revealed, setRevealed] = useState(false)

  useEffect(() => {
    if (!revealed) return
    const timer = setTimeout(() => setRevealed(false), autoHideSeconds * 1000)
    return () => clearTimeout(timer)
  }, [revealed, autoHideSeconds])

  const toggle = useCallback(() => setRevealed((v) => !v), [])

  return { revealed, toggle }
}
