import { MapContainer, TileLayer } from 'react-leaflet'
import React, { useMemo } from 'react'
import useMapStore from '@/store/store'
import { useSuspenseQuery } from '@tanstack/react-query'
import { infoQuery } from '@/api/queries'
import L from 'leaflet'

export interface MapProps extends React.PropsWithChildren {
  width?: string
  height?: string
}

const Map = ({ width = '100%', height = 'calc(100dvh - 4.563rem)', children }: MapProps) => {
  const { data: appInfo } = useSuspenseQuery(infoQuery())

  const { center, zoom, maxZoom, minZoom } = useMapStore((state) => ({
    center: state.map.center,
    zoom: state.map.zoom,
    maxZoom: state.map.maxZoom,
    minZoom: state.map.minZoom,
  }))

  const time = useMemo(() => new Date().getTime(), [])

  return (
    <MapContainer
      key={time}
      preferCanvas
      className="z-0"
      zoomControl={false}
      style={{ width, height }}
      center={center}
      zoom={zoom}
      maxZoom={maxZoom}
      minZoom={minZoom}
      maxBounds={L.latLngBounds(
        [appInfo?.map?.bbox[0], appInfo.map.bbox[1]],
        [appInfo?.map?.bbox[2], appInfo.map.bbox[3]],
      )}
    >
      <TileLayer
        attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      {children}
    </MapContainer>
  )
}

export default Map
