import L from 'leaflet'
import { SVG_CACHE } from './markerSvgCache'

const iconCache = new Map<string, L.DivIcon>()

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

export const TreeMarkerIcon = (color: string, isSelected: boolean, isHighlighted: boolean) => {
  const key = `tree-${color}-${isSelected}-${isHighlighted}`
  let icon = iconCache.get(key)
  if (!icon) {
    icon = L.divIcon({
      iconAnchor: [0, 24],
      popupAnchor: [0, -36],
      html: `<figure style="${makerWrapperStyles(isSelected, isHighlighted)}">
        <span style="${markerHtmlStyles(color)}">
          ${isSelected ? SVG_CACHE.check : SVG_CACHE.tree}
        </span>
      </figure>`,
    })
    iconCache.set(key, icon)
  }
  return icon
}

export const ClusterIcon = (
  color: string,
  isHighlighted: boolean,
  isDisabled: boolean,
  includedTrees: number,
) => {
  const key = `cluster-${color}-${isHighlighted}-${isDisabled}-${includedTrees}`
  let icon = iconCache.get(key)
  if (!icon) {
    icon = L.divIcon({
      iconAnchor: [0, 24],
      popupAnchor: [0, -36],
      html: `<figure style="${makerClusterWrapperStyles(isHighlighted, isDisabled)}">
        <span style="${markerClusterHtmlStyles(color, isDisabled)}">
          ${includedTrees}
        </span>
      </figure>`,
    })
    iconCache.set(key, icon)
  }
  return icon
}

const sensorMarkerIconCached = L.divIcon({
  iconAnchor: [0, 24],
  html: `<figure style="${makerWrapperStyles(false, true)}">
        <span style="${markerHtmlStyles('#454545')}">
          ${SVG_CACHE.sensor}
        </span>
      </figure>`,
})

export const SensorMarkerIcon = () => sensorMarkerIconCached

export const RouteIcon = (label: string) => {
  const key = `route-${label}`
  let icon = iconCache.get(key)
  if (!icon) {
    icon = L.divIcon({
      iconAnchor: [12, 12],
      html: `<span style="${makerRouteWrapperStyles()}">
      ${label}
    </span>`,
    })
    iconCache.set(key, icon)
  }
  return icon
}

const refillIconCached = L.divIcon({
  iconAnchor: [12, 12],
  html: `<figure style="${makerWrapperStyles(false, false)}">
      <span style="${markerHtmlStyles('#454545')}">
        ${SVG_CACHE.paintBucket}
      </span>
    </figure>`,
})

export const RefillIcon = () => refillIconCached
