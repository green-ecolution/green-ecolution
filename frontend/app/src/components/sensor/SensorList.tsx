import { Sensor } from '@/api/backendApi'
import SensorCard from '../general/cards/SensorCard'

interface SensorListProps {
  data: Sensor[]
}

const SensorList = ({ data }: SensorListProps) => {
  return (
    <ul>
      {data?.length === 0 ? (
        <li className="text-center text-dark-600 mt-10">
          <p>Es wurden keine Sensoren gefunden.</p>
        </li>
      ) : (
        data?.map((sensor) => (
          <li key={sensor.id} className="mb-5 last:mb-0">
            <SensorCard sensor={sensor} />
          </li>
        ))
      )}
    </ul>
  )
}

export default SensorList
