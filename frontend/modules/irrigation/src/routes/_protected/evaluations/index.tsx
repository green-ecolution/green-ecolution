import { evaluationQuery } from '@/api/queries'
import StatusCardGrid from '@/components/general/StatusCardGrid'
import { StatusCard } from '@green-ecolution/ui'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { pendingLoading } from '@/lib/router'

export const Route = createFileRoute('/_protected/evaluations/')({
  component: Evaluation,
  pendingComponent: pendingLoading({ key: 'evaluation:list.loadingLabel' }),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(evaluationQuery()),
})

function Evaluation() {
  const { data } = useSuspenseQuery(evaluationQuery())
  const { t } = useTranslation('evaluation')

  return (
    <>
      <div className="container mt-6">
        <article className="2xl:w-4/5">
          <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
            {t('list.title')}
          </h1>
          <p>{t('list.description')}</p>
        </article>

        <section className="mt-16">
          <h2 className="font-lato font-bold text-2xl mb-4">{t('list.treeSectionTitle')}</h2>
          <StatusCardGrid>
            <li>
              <StatusCard
                label={t('list.clusterCountLabel')}
                value={t('list.clusterCountValue', { count: data.treeclusterCount })}
                description={t('list.clusterCountDescription')}
              />
            </li>
            <li>
              <StatusCard
                label={t('list.treeCountLabel')}
                value={t('list.treeCountValue', { count: data.treeCount })}
                description={t('list.treeCountDescription')}
              />
            </li>
            <li>
              <StatusCard
                label={t('list.sensorCountLabel')}
                value={t('list.sensorCountValue', { count: data.sensorCount })}
                description={t('list.sensorCountDescription')}
              />
            </li>
          </StatusCardGrid>
        </section>

        <section className="mt-16">
          <h2 className="font-lato font-bold text-2xl mb-4">
            {t('list.wateringPlanSectionTitle')}
          </h2>
          <StatusCardGrid>
            <li>
              <StatusCard
                label={t('list.wateringPlanCountLabel')}
                value={t('list.wateringPlanCountValue', { count: data.wateringPlanCount })}
                description={t('list.wateringPlanCountDescription')}
              />
            </li>
            <li>
              <StatusCard
                label={t('list.waterConsumptionLabel')}
                value={t('list.waterConsumptionValue', { liters: data.totalWaterConsumption })}
                description={t('list.waterConsumptionDescription')}
              />
            </li>
            <li>
              <StatusCard
                label={t('list.userCountLabel')}
                value={t('list.userCountValue', { count: data.userWateringPlanCount })}
                description={t('list.userCountDescription')}
              />
            </li>
          </StatusCardGrid>
        </section>

        <EvaluationList
          title={t('list.regionListTitle')}
          data={data.regionEvaluation.map((region) => ({
            name: region.name,
            wateringPlanCount: region.wateringPlanCount,
          }))}
          label={t('list.regionListLabel')}
        />

        <EvaluationList
          title={t('list.vehicleListTitle')}
          data={data.vehicleEvaluation.map((vehicle) => ({
            name: vehicle.numberPlate,
            wateringPlanCount: vehicle.wateringPlanCount,
          }))}
          label={t('list.vehicleListLabel')}
        />
      </div>
    </>
  )
}

interface EvaluationItem {
  name: string
  wateringPlanCount: number
}

interface EvaluationListProps {
  title: string
  data: EvaluationItem[]
  label: string
}

const EvaluationList: React.FC<EvaluationListProps> = ({ title, data, label }) => {
  const { t } = useTranslation('evaluation')

  return (
    <section className="mt-16">
      <h2 className="font-lato font-bold text-2xl mb-4">{title}</h2>
      <header className="hidden border-b pb-2 text-sm text-dark-800 border-b-dark-200 sm:flex items-center justify-between">
        <p>{label}</p>
        <p>{t('list.wateringCountColumnHeader')}</p>
      </header>
      <ul className="flex flex-col gap-y-3 md:contents">
        {data.map((item) => (
          <li
            key={item.name}
            className="flex flex-wrap justify-between gap-3 items-center border-b border-b-dark-300 pb-3 sm:py-3"
          >
            <h3 className="font-bold text-lg">{item.name}</h3>
            <p>{t('list.wateringCountValue', { count: item.wateringPlanCount })}</p>
          </li>
        ))}
      </ul>
    </section>
  )
}
