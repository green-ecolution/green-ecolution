import React from 'react'

interface IntroductionCardProps {
  label: string
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>
  description: string
}

const IntroductionCard: React.FC<IntroductionCardProps> = ({ label, icon: Icon, description }) => {
  return (
    <div className="h-full cursor-pointer bg-white shadow-cards rounded-2xl p-6 border border-dark-50 xl:cursor-default">
      <figure className="bg-green-light-900/25 w-12 h-12 rounded-full flex items-center justify-center">
        {Icon && <Icon className="text-green-dark stroke-2" />}
      </figure>
      <h3 className="my-4 font-lato font-semibold text-lg md:my-5">{label}</h3>
      <p>{description}</p>
    </div>
  )
}

export default IntroductionCard
