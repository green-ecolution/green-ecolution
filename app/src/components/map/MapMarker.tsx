import defaultIconPng from 'leaflet/dist/images/marker-icon.png'
import L, { DivIcon, Icon, IconOptions } from 'leaflet'
import { Marker } from 'react-leaflet'
import { Marker as LeafletMarker } from 'leaflet'
import { useMemo, useRef } from 'react'
import { Check, PaintBucket } from 'lucide-react'
import { renderToStaticMarkup } from 'react-dom/server'
import TreeIcon from '../icons/Tree'
import SensorIcon from '../icons/Sensor'

const iconToSvg = (IconComponent: React.FC<React.SVGProps<SVGSVGElement>>) => {
  return renderToStaticMarkup(
    <IconComponent className="text-white w-[1.125rem] h-[1.125rem]" strokeWidth={3} />,
  )
}

const defaultIcon = new Icon({
  iconUrl: defaultIconPng,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  shadowSize: [41, 41],
})

export interface MapMarkerProps {
  icon: Icon<IconOptions> | DivIcon | undefined
  position: [number, number]
  onClick: () => void
}

const MapMarker = ({ position, icon, onClick }: MapMarkerProps) => {
  return (
    <Marker
      position={position}
      icon={icon}
      eventHandlers={{
        click: () => onClick(),
      }}
    ></Marker>
  )
}

const markerHtmlStyles = (color: string) => `
  background-color: ${color};
  width: 2rem;
  height: 2rem;
  position: absolute;
  border-radius: 3rem;
  left: 0.25rem;
  top: 0.25rem;
  border: 1px solid white;
  display: flex;
  align-items: center;
  justify-content: center;
`

const markerClusterHtmlStyles = (color: string, isDisabled: boolean) => `
  background-color: ${color};
  opacity: ${isDisabled ? '0.6' : '1'};
  cursor: ${isDisabled ? 'not-allowed' : 'pointer'};
  width: 2.25rem;
  height: 2.25rem;
  position: absolute;
  border-radius: 3rem;
  left: ${isDisabled ? '0;' : '0.25rem'};
  top: ${isDisabled ? '0;' : '0.25rem'};
  border: 1px solid white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 0.875rem;
  color: white;
  font-family: Nunito, sans-serif;
`

const makerWrapperStyles = (isSelected: boolean, isHighlighted: boolean) => `
  background-color: ${isSelected || isHighlighted ? 'white' : ''};
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 3rem;
  position: relative;
  left: -1rem;
  top: -1rem;
  box-shadow: rgba(0, 0, 0, ${isSelected || isHighlighted ? '0.35' : '0'}) 0px 5px 15px;
`

const makerClusterWrapperStyles = (isHighlighted: boolean, isDisabled: boolean) => `
  background-color: ${isHighlighted ? 'white' : isDisabled ? '#E8E8E8' : ''};
  width: ${isDisabled ? '2.25rem;' : '2.75rem'};
  height: ${isDisabled ? '2.25rem;' : '2.75rem'};
  border-radius: 3rem;
  position: relative;
  left: ${isDisabled ? '-1rem;' : '-1.25rem'};
  top: ${isDisabled ? '-1rem;' : '-1.25rem'};
  box-shadow: rgba(0, 0, 0, ${isHighlighted ? '0.35' : '0'}) 0px 5px 15px;
`

const makerRouteWrapperStyles = () => `
  background-color: #454545;
  width: 6rem;
  height: 1.75rem;
  position: absolute;
  border-radius: 3rem;
  left: 0.25rem;
  top: 0.25rem;
  border: 1px solid white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 0.875rem;
  color: white;
  z-index: 1500;
  font-family: Nunito, sans-serif;
`

export const TreeMarkerIcon = (color: string, isSelected: boolean, isHighlighted: boolean) =>
  L.divIcon({
    iconAnchor: [0, 24],
    popupAnchor: [0, -36],
    html: `<figure style="${makerWrapperStyles(isSelected, isHighlighted)}">
        <span style="${markerHtmlStyles(color)}">
          ${isSelected ? iconToSvg(Check) : iconToSvg(TreeIcon)}
        </span>
      </figure>`,
  })

export const ClusterIcon = (
  color: string,
  isHighlighted: boolean,
  isDisabled: boolean,
  includedTrees: number,
) =>
  L.divIcon({
    iconAnchor: [0, 24],
    popupAnchor: [0, -36],
    html: `<figure style="${makerClusterWrapperStyles(isHighlighted, isDisabled)}">
        <span style="${markerClusterHtmlStyles(color, isDisabled)}">
          ${includedTrees}
        </span>
      </figure>`,
  })

export const SensorMarkerIcon = () =>
  L.divIcon({
    iconAnchor: [0, 24],
    html: `<figure style="${makerWrapperStyles(false, true)}">
        <span style="${markerHtmlStyles('#454545')}">
          ${iconToSvg(SensorIcon)}
        </span>
      </figure>`,
  })

export const RouteIcon = (label: string) =>
  L.divIcon({
    iconAnchor: [12, 12],
    html: `<span style="${makerRouteWrapperStyles()}">
      ${label}
    </span>`,
  })

export const RefillIcon = () =>
  L.divIcon({
    iconAnchor: [12, 12],
    html: `<figure style="${makerWrapperStyles(false, false)}">
      <span style="${markerHtmlStyles('#454545')}">
        ${iconToSvg(PaintBucket)}
      </span>
    </figure>`,
  })

interface DragableMarkerProps {
  position: L.LatLng
  onMove?: (latlng: L.LatLng) => void
  onDrag?: (latlng: L.LatLng) => void
}

export const DragableMarker = ({ position, onDrag, onMove }: DragableMarkerProps) => {
  const markerRef = useRef<LeafletMarker>(null)
  const eventHandlers = useMemo(
    () => ({
      move() {
        const marker = markerRef.current
        if (marker != null) {
          onMove?.(marker.getLatLng())
        }
      },
      dragend() {
        const marker = markerRef.current
        if (marker != null) {
          onDrag?.(marker.getLatLng())
        }
      },
    }),
    [onDrag, onMove],
  )

  return (
    <Marker
      eventHandlers={eventHandlers}
      icon={defaultIcon}
      ref={markerRef}
      draggable
      position={position}
    />
  )
}

export default MapMarker
