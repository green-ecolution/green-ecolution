import { ReactNode } from '@tanstack/react-router'
import React, { useState } from 'react'

interface Tabs {
  tabs: { label: string; icon: ReactNode; view: ReactNode }[]
}

const Tabs: React.FC<Tabs> = ({ tabs }) => {
  const [showTabIndex, setShowTabIndex] = useState(0)

  return (
    <>
      <div
        role="tablist"
        className="mt-10 mb-6 border-b border-b-dark-600 flex items-center w-max gap-x-6"
      >
        {tabs.map((tab, key) => (
          <button
            onClick={() => setShowTabIndex(key)}
            type="button"
            key={tab.label}
            id={`tab-${key}`}
            role="tab"
            aria-selected={showTabIndex === key}
            aria-controls={`tabpanel-${key}`}
            className={`flex items-center gap-x-2 pb-2 group border-b transition-all ease-in-out duration-300 hover:text-dark-800 ${showTabIndex === key ? 'text-dark border-b-dark' : 'text-dark-600 border-b-transparent'}`}
          >
            {tab.icon}
            <span className="hidden group-aria-selected:block lg:block">{tab.label}</span>
          </button>
        ))}
      </div>

      {tabs.map((tab, key) =>
        showTabIndex === key ? (
          <div
            key={tab.label}
            id={`tabpanel-${key}`}
            role="tabpanel"
            tabIndex={0}
            aria-labelledby={`tab-${key}`}
          >
            {tab.view}
          </div>
        ) : null,
      )}
    </>
  )
}

export default Tabs
