import { Ref } from 'react'

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label: string
  error?: string
  ref?: Ref<HTMLTextAreaElement>
}

const Textarea = ({ ref, ...props }: TextareaProps) => {
  return (
    <div>
      <label htmlFor={props.name} className="block font-semibold text-dark-800 mb-2.5">
        {props.label} {props.required ? <span className="text-red">*</span> : null}
      </label>
      <textarea
        ref={ref}
        rows={4}
        cols={40}
        className="w-full text-dark-800 border border-green-light rounded-lg bg-white px-4 py-3 focus:outline-green-dark"
        {...props}
      />
      {props.error && (
        <span className="block text-red mt-2 font-semibold text-sm">{props.error}</span>
      )}
    </div>
  )
}

export default Textarea
