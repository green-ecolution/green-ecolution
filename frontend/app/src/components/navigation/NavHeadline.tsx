import React from 'react'

interface NavHeadline {
  label: string
  collapsed?: boolean
}

const NavHeadline: React.FC<NavHeadline> = ({ label, collapsed = false }) => {
  if (!label) return null

  return (
    <p
      className={`mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-dark-400 ${collapsed ? 'lg:hidden' : ''}`}
    >
      {label}
    </p>
  )
}

export default NavHeadline
