import { Link } from '@tanstack/react-router'
import { X } from 'lucide-react'
import React from 'react'

interface NavHeader {
  isOpen: boolean
  closeSidebar: () => void
}

const NavHeader: React.FC<NavHeader> = ({ isOpen, closeSidebar }) => {
  return (
    <div className="relative mb-10 flex items-center justify-between">
      <Link
        to="/dashboard"
        className="block transition-all ease-in-out duration-300 hover:opacity-75"
        aria-label="Zurück zum persönlichen Dashboard"
        onClick={closeSidebar}
      >
        <img
          className="h-4 lg:hidden"
          src="/images/logo/logo-large-white.svg"
          alt="Logo von Green Ecolution"
        />
        <img
          className="hidden w-7 mx-2.5 lg:block"
          src="/images/logo/logo-icon-white.svg"
          alt="Logo von Green Ecolution"
        />
      </Link>
      <button
        type="button"
        aria-label="Hauptnavigation schließen"
        className={`mr-2 w-8 h-8 flex items-center justify-center transition-colors ease-in-out duration-300 bg-dark-600 rounded-full lg:hidden 
            ${isOpen ? 'lg:flex' : 'lg:hidden'}`}
        onClick={closeSidebar}
      >
        <X className="w-5 h-5 text-white" />
      </button>
    </div>
  )
}

export default NavHeader
