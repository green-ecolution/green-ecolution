import { TriangleAlert } from 'lucide-react'
import React from 'react'

interface NoticeProps {
  description: string
  classes?: string
}

const Notice: React.FC<NoticeProps> = ({ description, classes }) => {
  return (
    <div className={`bg-red-100 w-fit p-4 rounded-2xl flex items-center gap-x-2 ${classes}`}>
      <TriangleAlert className="flex-shrink-0 text-red size-6" />
      <p className="ml-2 text-red text-sm font-semibold">{description}</p>
    </div>
  )
}

export default Notice
