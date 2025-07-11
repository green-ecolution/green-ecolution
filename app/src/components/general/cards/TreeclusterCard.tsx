import TreeIcon from '@/components/icons/Tree'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { TreeClusterInList } from '@green-ecolution/backend-client'
import { Link } from '@tanstack/react-router'
import { MapPin } from 'lucide-react'
import React from 'react'

interface TreeclusterCard {
  treecluster: TreeClusterInList
}

const TreeclusterCard: React.FC<TreeclusterCard> = ({ treecluster }) => {
  const statusDetails = getWateringStatusDetails(treecluster.wateringStatus)

  return (
    <Link
      to={`/treecluster/$treeclusterId`}
      params={{
        treeclusterId: treecluster.id.toString(),
      }}
      className="bg-white border border-dark-50 p-6 rounded-xl shadow-cards flex flex-col gap-y-4 transition-all ease-in-out duration-300 hover:bg-green-dark-50 hover:border-green-dark lg:grid lg:grid-cols-[1fr,2fr,1.5fr,1fr] lg:items-center lg:gap-5 lg:py-10 xl:px-10"
    >
      <p
        className={`relative font-medium pl-7 before:absolute before:w-4 before:h-4 before:rounded-full before:left-0 before:top-[0.22rem] 
          before:bg-${statusDetails.color}`}
      >
        {statusDetails.label}
      </p>

      <h2 className="font-bold text-lg mb-0.5">{treecluster.name}</h2>

      <div className="text-dark-800 flex gap-x-2">
        <MapPin className="w-5 h-5" />
        <p>
          <span>{treecluster.address}, </span>
          <br />
          <span className="text-dark-600 lg:block lg:text-sm">
            {treecluster.region?.name ?? '-'}
          </span>
        </p>
      </div>

      <div className="text-dark-800 flex gap-x-2">
        <TreeIcon className="w-5 h-5 mt-0.5" />
        <p>
          {treecluster.treeIds ? treecluster.treeIds?.length : 0}
          {treecluster.treeIds?.length === 1 ? ' Baum' : ' Bäume'}
        </p>
      </div>
    </Link>
  )
}

export default TreeclusterCard
