import { SVGProps } from 'react'

// Mini KA5 diagram: right triangle (right angle bottom-left) with a marker dot,
// drawn lucide-style so it blends with the icon set.
const SoilTriangleIcon = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
    {...props}
  >
    <path d="M5 4v16h16z" />
    <circle cx="10" cy="15" r="1" fill="currentColor" stroke="none" />
  </svg>
)

export default SoilTriangleIcon
