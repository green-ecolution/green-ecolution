import BackLink from '@/components/general/links/BackLink'
import SensorAbilitiesSection from './detail/SensorAbilitiesSection'
import SensorActionsProvider from './detail/SensorActionsContext'
import SensorHero from './detail/SensorHero'
import SensorIdentitySection from './detail/SensorIdentitySection'
import SensorLinkedTreeSection from './detail/SensorLinkedTreeSection'
import SensorLocationSection from './detail/SensorLocationSection'
import SensorLorawanConfigSection from './detail/SensorLorawanConfigSection'
import SensorStatusGrid from './detail/SensorStatusGrid'
import type { Sensor } from '@/api/backendApi'

interface SensorDashboardProps {
  sensor: Sensor
}

const SensorDashboard = ({ sensor }: SensorDashboardProps) => {
  return (
    <SensorActionsProvider sensor={sensor}>
      <BackLink link={{ to: '/sensors', search: { page: 1 } }} label="Zu allen Sensoren" />
      <div className="flex flex-col gap-10 pb-16">
        <SensorHero sensor={sensor} />
        <SensorStatusGrid sensor={sensor} />
        <SensorIdentitySection sensor={sensor} />
        <SensorAbilitiesSection sensor={sensor} />
        <SensorLocationSection sensor={sensor} />
        <SensorLinkedTreeSection sensor={sensor} />
        <SensorLorawanConfigSection sensor={sensor} />
      </div>
    </SensorActionsProvider>
  )
}

export default SensorDashboard
