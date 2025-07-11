import React from 'react'

interface DetailedListProps {
  headline?: string
  details: { label: string; value: string }[]
  hasNoGrid?: boolean
}

const DetailedList: React.FC<DetailedListProps> = ({ headline, details, hasNoGrid = false }) => {
  return (
    <>
      {headline && <h2 className="font-lato font-bold text-2xl mb-4">{headline}</h2>}
      <dl className={`text-lg ${hasNoGrid ? '' : 'md:columns-2 md:gap-x-11'}`}>
        {details.map((data, index) => (
          <div
            key={data.label}
            className={`py-4 border-b border-b-dark-200 group md:last:border-b-transparent
               ${
                 hasNoGrid
                   ? 'last:border-b-transparent'
                   : `${details.length / 2 === index + 1 ? 'md:border-b-transparent' : ''}`
               }`}
          >
            <dt className="font-bold sm:inline">{data.label}:</dt>
            <dd className="sm:inline sm:px-2">{data.value}</dd>
          </div>
        ))}
      </dl>
    </>
  )
}

export default DetailedList
