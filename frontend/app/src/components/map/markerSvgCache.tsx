import { Check, PaintBucket } from 'lucide-react'
import { renderToStaticMarkup } from 'react-dom/server'
import TreeIcon from '../icons/Tree'
import SensorIcon from '../icons/Sensor'

export const SVG_CACHE = {
  tree: renderToStaticMarkup(
    <TreeIcon className="text-white w-[1.125rem] h-[1.125rem]" strokeWidth={3} />,
  ),
  check: renderToStaticMarkup(
    <Check className="text-white w-[1.125rem] h-[1.125rem]" strokeWidth={3} />,
  ),
  sensor: renderToStaticMarkup(
    <SensorIcon className="text-white w-[1.125rem] h-[1.125rem]" strokeWidth={3} />,
  ),
  paintBucket: renderToStaticMarkup(
    <PaintBucket className="text-white w-[1.125rem] h-[1.125rem]" strokeWidth={3} />,
  ),
}
