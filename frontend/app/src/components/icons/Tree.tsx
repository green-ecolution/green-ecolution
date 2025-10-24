import React from 'react'

const TreeIcon: React.FC<React.SVGProps<SVGSVGElement>> = (props) => (
  <svg
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    {...props}
  >
    <path
      d="M3.5607 3.69226C10.4781 3.69246 12 6.66426 12 10.3114C12 6.66426 13.5219 3.69246 20.4393 3.69226C20.4017 7.31032 18.8328 11.5272 12 11.5272C5.16719 11.5272 3.59831 7.31032 3.5607 3.69226Z"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinejoin="round"
    />
    <path
      d="M3.5607 12.6079C10.4781 12.6081 12 15.5799 12 19.2271C12 15.5799 13.5219 12.6081 20.4393 12.6079C20.4017 16.2259 18.8328 20.4428 12 20.4428C5.16719 20.4428 3.59831 16.2259 3.5607 12.6079Z"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinejoin="round"
    />
  </svg>
)

export default TreeIcon
