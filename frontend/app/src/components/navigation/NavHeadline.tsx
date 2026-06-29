import React from 'react'

interface NavHeadline {
  label: string
}

const NavHeadline: React.FC<NavHeadline> = ({ label }) => {
  if (!label) return null

  return (
    <p className="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-dark-400 lg:hidden xl:block">
      {label}
    </p>
  )
}

export default NavHeadline
