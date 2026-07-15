import { SoilCondition } from '@green-ecolution/backend-client'
import {
  SOIL_REGIONS,
  SoilGroup,
  SoilRegion,
  polygonCentroid,
  regionPolygon,
} from '@/lib/soilTexture'

const GROUP_FILL: Record<SoilGroup, string> = {
  sand: 'var(--soil-sand)',
  silt: 'var(--soil-silt)',
  loam: 'var(--soil-loam)',
  clay: 'var(--soil-clay)',
}

const PLOT = { x: 14, y: 6, size: 100 }
const toX = (silt: number) => PLOT.x + silt
const toY = (clay: number) => PLOT.y + (PLOT.size - clay)
const TICKS = [0, 25, 50, 75, 100]

interface SoilTextureTriangleProps {
  silt: number
  clay: number
  activeCondition: SoilCondition
}

const Region = ({ region, active }: { region: SoilRegion; active: boolean }) => {
  const polygon = regionPolygon(region)
  const centroid = polygonCentroid(polygon)
  const points = polygon.map((p) => `${toX(p.silt)},${toY(p.clay)}`).join(' ')

  return (
    <g>
      <polygon
        data-testid={`region-${region.condition}`}
        data-active={active}
        points={points}
        fill={GROUP_FILL[region.group]}
        fillOpacity={active ? 1 : 0.55}
        strokeWidth={active ? 0.8 : 0.3}
        className={active ? 'stroke-ring' : 'stroke-border'}
      />
      <text
        x={toX(centroid.silt)}
        y={toY(centroid.clay)}
        fontSize={3.2}
        textAnchor="middle"
        dominantBaseline="central"
        className={active ? 'fill-foreground font-semibold' : 'fill-muted-foreground'}
      >
        {region.condition}
      </text>
    </g>
  )
}

const SoilTextureTriangle = ({ silt, clay, activeCondition }: SoilTextureTriangleProps) => {
  const activeRegion = SOIL_REGIONS.find((r) => r.condition === activeCondition)
  const inactiveRegions = SOIL_REGIONS.filter((r) => r.condition !== activeCondition)

  return (
    <svg
      viewBox="0 0 126 122"
      role="img"
      aria-label={`Bodenartendiagramm: Schluff ${silt} %, Ton ${clay} %, Klasse ${activeCondition}`}
      className="w-full select-none"
    >
      {inactiveRegions.map((region) => (
        <Region key={region.condition} region={region} active={false} />
      ))}
      {activeRegion && <Region region={activeRegion} active />}

      {TICKS.map((tick) => (
        <g key={tick} className="fill-muted-foreground" fontSize={3}>
          <text x={toX(tick)} y={PLOT.y + PLOT.size + 5} textAnchor="middle">
            {tick}
          </text>
          <text x={PLOT.x - 2.5} y={toY(tick)} textAnchor="end" dominantBaseline="central">
            {tick}
          </text>
        </g>
      ))}
      <text
        x={toX(50)}
        y={PLOT.y + PLOT.size + 11}
        textAnchor="middle"
        fontSize={3.5}
        className="fill-foreground"
      >
        Schluff [%]
      </text>
      <text
        x={3.5}
        y={toY(50)}
        textAnchor="middle"
        fontSize={3.5}
        className="fill-foreground"
        transform={`rotate(-90 3.5 ${toY(50)})`}
      >
        Ton [%]
      </text>

      {/* transition must sit on the circles: cx/cy change there, not on a wrapper */}
      <circle
        data-testid="soil-marker"
        cx={toX(silt)}
        cy={toY(clay)}
        r={2}
        className="fill-ring transition-all duration-150 motion-reduce:transition-none"
      />
      <circle
        cx={toX(silt)}
        cy={toY(clay)}
        r={0.7}
        className="fill-background transition-all duration-150 motion-reduce:transition-none"
      />
    </svg>
  )
}

export default SoilTextureTriangle
