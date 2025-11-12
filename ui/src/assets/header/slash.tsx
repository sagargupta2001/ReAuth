import { type SVGProps } from 'react'

export function Slash(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="16"
      height="16"
      stroke="currentColor"
      stroke-width="1"
      stroke-linecap="round"
      stroke-linejoin="round"
      fill="none"
      shape-rendering="geometricPrecision"
      data-sentry-element="svg"
      data-sentry-source-file="LayoutHeader.tsx"
      {...props}
    >
      <path
        d="M16 3.549L7.12 20.600"
        data-sentry-element="path"
        data-sentry-source-file="LayoutHeader.tsx"
      ></path>
    </svg>
  )
}
