import { evaluationQuery } from '@/api/queries'
import GeneralStatusCard from '@/components/general/cards/GeneralStatusCard'
import LoadingInfo from '@/components/general/error/LoadingInfo'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/evaluations/')({
  component: Evaluation,
  pendingComponent: () => <LoadingInfo label="Auswertungen werden geladen" />,
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(evaluationQuery()),
})

function Evaluation() {
  const { data } = useSuspenseQuery(evaluationQuery())

  return (
    <>
      <div className="container mt-6">
        <article className="2xl:w-4/5">
          <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
            Auswertung aller Daten
          </h1>
          <p>
            Diese Seite bietet eine detaillierte Auswertung aller relevanten Daten, die für die
            Verwaltung von Bewässerungs- und Einsatzplänen sowie für die Wartung von Bäumen und
            Sensoren erforderlich sind. Es wird aufgezeigt, wie viele Einsatzfahrten im System
            hinterlegt worden sind, welche Fahrzeuge die meisten Einsatzfahrten durchführen und
            welche Stadtteile am meisten bewässert werden. Außerdem können Sie sich darüber
            informieren, wie viele Bäume, Bewässerungsgruppen und Sensoren erstellt worden sind.
          </p>
        </article>

        <section className="mt-16">
          <h2 className="font-lato font-bold text-2xl mb-4">
            Erhobene Daten zu Bäumen, Bewässerungsgruppen & Sensoren:
          </h2>
          <ul className="space-y-5 md:space-y-0 md:grid md:gap-5 md:grid-cols-2 lg:grid-cols-3">
            <li>
              <GeneralStatusCard
                overline="Anzahl an Bewässerungsgruppen"
                value={`${data.treeclusterCount} ${data.treeclusterCount === 1 ? 'Gruppe' : 'Gruppen'}`}
                description="Die Bewässerungsgruppen bilden Gruppen von Bäumen ab mit gleichen Standortbedingungen."
              />
            </li>
            <li>
              <GeneralStatusCard
                overline="Anzahl an Bäumen"
                value={`${data.treeCount} ${data.treeCount === 1 ? 'Baum' : 'Bäume'}`}
                description="inkl. manuell erstellte oder importierte Bäume aus anderen Systemen (z.B. Datenbank)."
              />
            </li>
            <li>
              <GeneralStatusCard
                overline="Anzahl an Sensoren"
                value={`${data.sensorCount} ${data.sensorCount === 1 ? 'Sensor' : 'Sensoren'}`}
                description="Es werden auch Sensoren einberechnet, die nicht mit einer Vegetation verknüpft sind.."
              />
            </li>
          </ul>
        </section>

        <section className="mt-16">
          <h2 className="font-lato font-bold text-2xl mb-4">
            Erhobene Daten zu den Einsatzplänen:
          </h2>
          <ul className="space-y-5 md:space-y-0 md:grid md:gap-5 md:grid-cols-2 lg:grid-cols-3">
            <li>
              <GeneralStatusCard
                overline="Anzahl an Einsatzfahrten"
                value={`${data.wateringPlanCount} ${data.wateringPlanCount === 1 ? 'Fahrt' : 'Fahrten'}`}
                description="Dies beschreibt die Anzahl aller Einsatzfahrten, die aktuell im System hinterlegt sind."
              />
            </li>
            <li>
              <GeneralStatusCard
                overline="Wasserverbrauch"
                value={`${data.totalWaterConsumption} Liter`}
                description="Der Wasserverbrauch bezieht sich auf alle Einsatzfahrten, die in dem angegebenen Zeitraum durchgeführt worden sind."
              />
            </li>
            <li>
              <GeneralStatusCard
                overline="Anzahl Mitarbeitende für Einsatzpläne"
                value={`${data.userWateringPlanCount} Mitarbeitende`}
                description="Dies beschreibt die Anzahl aller Mitarbeitenden, die zu Einsatzplänen verlinkt wurden. Dopplungen wurden nicht exkludiert."
              />
            </li>
          </ul>
        </section>

        <EvaluationList
          title="Anzahl an Bewässerungen pro Stadtteil:"
          data={data.regionEvaluation.map((region) => ({
            name: region.name,
            wateringPlanCount: region.wateringPlanCount,
          }))}
          label="Stadtteil"
        />

        <EvaluationList
          title="Nutzung der Fahrzeuge:"
          data={data.vehicleEvaluation.map((vehicle) => ({
            name: vehicle.numberPlate,
            wateringPlanCount: vehicle.wateringPlanCount,
          }))}
          label="Fahrzeug"
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
  return (
    <section className="mt-16">
      <h2 className="font-lato font-bold text-2xl mb-4">{title}</h2>
      <header className="hidden border-b pb-2 text-sm text-dark-800 border-b-dark-200 sm:flex items-center justify-between">
        <p>{label}</p>
        <p>Anzahl der Bewässerungen</p>
      </header>
      <ul className="space-y-3 md:space-y-0">
        {data.map((item) => (
          <li
            key={item.name}
            className="flex flex-wrap justify-between gap-3 items-center border-b border-b-dark-300 pb-3 sm:py-3"
          >
            <h3 className="font-bold text-lg">{item.name}</h3>
            <p>
              {item.wateringPlanCount}{' '}
              {item.wateringPlanCount === 1 ? 'Bewässerung' : 'Bewässerungen'}
            </p>
          </li>
        ))}
      </ul>
    </section>
  )
}
