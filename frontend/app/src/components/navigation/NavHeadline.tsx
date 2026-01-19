import React from 'react'
import { useMediaQuery } from '@/hooks/useMediaQuery'

interface NavHeadline {
  label: string
  navIsOpen?: boolean
}

const NavHeadline: React.FC<NavHeadline> = ({ label, navIsOpen = false }) => {
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const showSeparator = isLargeScreen && !navIsOpen

  return (
    <p
      className={`mb-3 font-bold text-sm text-dark-400 tracking-[0.3] ${showSeparator ? 'border-t border-dark-400/30 pb-5' : ''}`}
    >
      <span
        className="transition-opacity ease-in-out duration-300"
        style={showSeparator ? { opacity: 0, display: 'none' } : { opacity: 1 }}
      >
        {label}
      </span>
    </p>
  )
}

export default NavHeadline
