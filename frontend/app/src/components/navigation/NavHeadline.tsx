import React from 'react'

interface NavHeadline {
  label: string
  navIsOpen?: boolean
}

const NavHeadline: React.FC<NavHeadline> = ({ label, navIsOpen = false }) => {
  return (
    <p
      className={`mb-3 font-bold text-sm text-dark-400 tracking-[0.3]
        ${navIsOpen ? '' : 'lg:border-t lg:border-t-dark-100/30 lg:pb-[1.17rem]'}`}
    >
      <span
        className={`opacity-full transition-opacity ease-in-out duration-300 
          ${navIsOpen ? 'lg:block' : 'lg:opacity-0 lg:hidden'}`}
      >
        {label}
      </span>
    </p>
  )
}

export default NavHeadline
