import { useEffect, useRef, RefObject } from 'react'

export const useOutsideClick = <T extends HTMLElement = HTMLElement>(
  callback: (event: MouseEvent) => void,
): RefObject<T | null> => {
  const ref = useRef<T | null>(null)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        callback(event)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [callback])

  return ref
}

export default useOutsideClick
