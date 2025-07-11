import { WateringStatus, Tree } from '@green-ecolution/backend-client'
import React from 'react'
import { TreeDeciduous } from 'lucide-react'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import GeneralStatusCard from '../general/cards/GeneralStatusCard'
import EntitiesStatusCard from '../general/cards/EntitiesStatusCard'
import ChartWateringData from './ChartWateringData'
import { format } from 'date-fns'
import { roundTo } from '@/lib/utils'

interface TabWateringStatusProps {
  tree?: Tree
}

const TabWateringStatus: React.FC<TabWateringStatusProps> = ({ tree }) => {
  const statusCards = [
    {
      id: 1,
      overline: 'Bodenfeuchte',
      value: tree?.sensor?.latestData
        ? `${roundTo(tree.sensor.latestData.humidity, 2)} %`
        : 'Keine Daten',
      isLarge: true,
      description: 'Wert bezeichnet den Wassergehalt im Boden.',
    },
    {
      id: 2,
      overline: 'Bodentemperatur',
      value: tree?.sensor?.latestData
        ? `${roundTo(tree?.sensor?.latestData.temperature, 2)} °C`
        : 'Keine Daten',
      isLarge: true,
      description: 'Wert bezeichnet die Temperatur in der oberflächlichen Bodenschicht.',
    },
    {
      id: 3,
      overline: 'Datum der letzten Bewässerung',
      value: tree?.lastWatered ? format(new Date(tree.lastWatered), 'dd.MM.yyyy') : 'Keine Angabe',
      isLarge: false,
      description:
        'Wird aktualisiert, sobald ein Einsatzplan mit diesem Baum als »Beendet« markiert wird.',
    },
  ]

  return (
    <>
      <ul className="space-y-5 md:space-y-0 md:grid md:gap-5 md:grid-cols-2 lg:grid-cols-4">
        <li>
          <EntitiesStatusCard
            statusDetails={getWateringStatusDetails(
              tree?.wateringStatus ?? WateringStatus.WateringStatusUnknown,
            )}
            label="Bewässerungszustand (ø)"
          />
        </li>
        {statusCards.map((card) => (
          <li key={card.id}>
            <GeneralStatusCard
              overline={card.overline}
              value={card.value}
              isLarge={card.isLarge}
              description={card.description}
            />
          </li>
        ))}
      </ul>

      {tree?.sensor?.latestData && (
        <section className="mt-16">
          <h2 className="font-bold font-lato text-xl mb-4">
            Aktuelle Bewässerungszustand pro Bodentiefe
          </h2>
          <p className="mb-6 lg:mb-0">
            Es werden drei Sensoren in 30 cm, 60 cm und 90 cm Tiefe in der Nähe des Wurzelballens
            angebracht. Diese Sensoren messen in jeder Tiefe das Bodenwasserpotential, das in
            Zentibar (cb) angegeben wird. Je besser der Baum bewässert ist, desto geringer ist der
            Aufwand, den er betreiben muss, um Wasser aus dem Boden aufzunehmen.
          </p>

          <div className="lg:grid lg:grid-cols-[auto,15rem] lg:items-end lg:gap-x-10 xl:gap-x-20 xl:grid-cols-[auto,20rem]">
            <div aria-hidden="true" className="mb-10 lg:mb-0 lg:w-60 lg:col-start-2 xl:w-80">
              <TreeDeciduous className="w-11 h-11 mx-auto mb-4" />
              <ul className="space-y-3">
                {tree?.sensor.latestData.watermarks.map((watermark) => (
                  <li key={watermark.depth} className={`rounded-xl text-center py-3 bg-dark-50`}>
                    <p className={`inline relative pl-8`}>
                      <span className="font-semibold">{watermark.centibar} Zentibar</span>
                      <span className="text-dark-800 font-normal">
                        &nbsp;·&nbsp;
                        {watermark.depth} cm
                      </span>
                    </p>
                  </li>
                ))}
              </ul>
            </div>

            <div className="lg:col-start-1 lg:row-start-1">
              <header className="hidden border-b pb-2 text-sm text-dark-800 border-b-dark-200 lg:grid lg:grid-cols-3 lg:gap-x-5">
                <p>Eingesetzte Tiefe</p>
                <p>Zentibar</p>
                <p>Ohmscher Widerstand</p>
              </header>

              <ul className="space-y-3 lg:space-y-0">
                {tree?.sensor.latestData.watermarks.map((watermark) => (
                  <li
                    key={watermark.depth}
                    className="space-y-3 border-b border-b-dark-300 pb-3 lg:py-3 lg:space-y-0 lg:grid lg:grid-cols-3 lg:gap-5"
                  >
                    <h3 className="font-medium text-lg">
                      <span className="lg:hidden">Bodentiefe von </span>
                      {watermark.depth} cm
                    </h3>
                    <div>
                      <dt className="inline lg:hidden">Wert in Zentibar:</dt>
                      <dd className="inline">{watermark.centibar} cb</dd>
                    </div>
                    <div>
                      <dt className="inline lg:hidden">Ohmscher Widerstand:</dt>
                      <dd className="inline">{watermark.resistance} Ω</dd>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </section>
      )}

      {tree?.sensor && (
        <section className="mt-16">
          <h2 className="font-bold font-lato text-xl mb-4">Messwerte im Verlaufe der Zeit</h2>
          <p className="mb-6">
            In diesem Abschnitt werden alle gemessenen Sensorwerte ausgegeben, die im System
            abgespeichert wurden.
            <br />
            Anhand dessen kann nachvollzogen werden, wie sich der Bewässerungszustand im Laufe der
            Zeit geändert hat.
          </p>

          <ChartWateringData sensorId={tree.sensor.id} />
        </section>
      )}
    </>
  )
}

export default TabWateringStatus
