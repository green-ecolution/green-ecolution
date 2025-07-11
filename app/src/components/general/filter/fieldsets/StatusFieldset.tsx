import { useFilter } from '@/context/FilterContext'
import Option from '../Option'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { WateringStatus } from '@green-ecolution/backend-client'

const StatusFieldset = () => {
  const { filters, handleStatusChange } = useFilter()
  return (
    <fieldset>
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Zustand der Bewässerung:
      </legend>
      {Object.entries(WateringStatus).map(([, statusValue]) => (
        <Option
          key={statusValue}
          value={statusValue}
          label={getWateringStatusDetails(statusValue).label}
          name={statusValue}
          checked={filters.statusTags.includes(statusValue)}
          onChange={handleStatusChange}
        >
          <div
            className={`bg-${getWateringStatusDetails(statusValue).color} w-4 h-4 rounded-full`}
          />
        </Option>
      ))}
    </fieldset>
  )
}

export default StatusFieldset
