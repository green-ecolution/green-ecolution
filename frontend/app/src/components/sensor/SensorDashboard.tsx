import { useTranslation } from 'react-i18next'
import BackLink from '@/components/general/links/BackLink'
import SensorActionsProvider from './detail/SensorActionsContext'
import SensorDataQualitySection from './detail/SensorDataQualitySection'
import SensorHero from './detail/SensorHero'
import SensorIdentitySection from './detail/SensorIdentitySection'
import SensorKpiRow from './detail/SensorKpiRow'
import SensorLorawanConfigSection from './detail/SensorLorawanConfigSection'
import SensorMeasurementProfile from './detail/SensorMeasurementProfile'
import SensorPlacementCard from './detail/SensorPlacementCard'
import SensorSignalCard from './detail/SensorSignalCard'
import SensorSoilMoistureChart from './detail/SensorSoilMoistureChart'
import type { Sensor } from '@/api/backendApi'

interface SensorDashboardProps {
  sensor: Sensor
}

const SensorDashboard = ({ sensor }: SensorDashboardProps) => {
  const { t } = useTranslation('sensor')
  return (
    <SensorActionsProvider sensor={sensor}>
      <BackLink link={{ to: '/sensors', search: { page: 1 } }} label={t('detail.backToList')} />
      <div className="flex flex-col gap-4 pb-16 md:gap-6">
        <SensorHero sensor={sensor} />
        <SensorDataQualitySection sensorId={sensor.id} />
        <SensorKpiRow sensor={sensor} />

        {/* Below xl the main column dissolves (`contents`) so `order` can slot the
            placement right after the chart, where a phone user needs it.
            min-w-0 keeps the Recharts svg from locking the page wider. */}
        <div className="flex flex-col gap-4 md:gap-6 xl:grid xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)] xl:items-start">
          <div className="contents xl:flex xl:min-w-0 xl:flex-col xl:gap-6">
            <div className="order-1 min-w-0 empty:hidden xl:order-none">
              <SensorSoilMoistureChart sensor={sensor} />
            </div>
            <div className="order-3 min-w-0 empty:hidden xl:order-none">
              <SensorSignalCard sensor={sensor} />
            </div>
            <div className="order-4 min-w-0 empty:hidden xl:order-none">
              <SensorIdentitySection sensor={sensor} />
            </div>
            <div className="order-5 min-w-0 empty:hidden xl:order-none">
              <SensorLorawanConfigSection sensor={sensor} />
            </div>
          </div>
          {/* auto-fit: side by side on a tablet, a single column in the narrow
              xl sidebar, and full width if the profile renders nothing. */}
          <div className="order-2 grid min-w-0 gap-4 md:grid-cols-[repeat(auto-fit,minmax(18rem,1fr))] md:gap-6 xl:order-none">
            <SensorPlacementCard sensor={sensor} />
            <SensorMeasurementProfile sensor={sensor} />
          </div>
        </div>
      </div>
    </SensorActionsProvider>
  )
}

export default SensorDashboard
