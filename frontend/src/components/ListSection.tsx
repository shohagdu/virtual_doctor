import { useState } from 'react'

/**
 * A section heading with ➕ and a list of items with red ✕ — the building block of the
 * prescription screen's left panel.
 */
export function ListSection({
  title,
  items,
  onChange,
  placeholder,
  readOnly = false,
  suggestions,
}: {
  title: string
  items: string[]
  onChange?: (items: string[]) => void
  placeholder?: string
  readOnly?: boolean
  suggestions?: string[]
}) {
  const [adding, setAdding] = useState(false)
  const [text, setText] = useState('')
  const listId = suggestions ? `dl-${title.replace(/\W/g, '')}` : undefined

  const add = () => {
    const v = text.trim()
    if (v && !items.includes(v)) onChange?.([...items, v])
    setText('')
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-sm font-semibold">
        <span>{title}</span>
        {!readOnly && (
          <button
            type="button"
            onClick={() => setAdding(!adding)}
            className="text-brand"
            aria-label={`Add ${title}`}
          >
            ➕
          </button>
        )}
      </div>
      {items.length === 0 && !adding && <div className="text-xs text-slate-400">—</div>}
      <ul className="space-y-0.5">
        {items.map((item, i) => (
          <li key={item} className="group flex items-start justify-between gap-2 text-sm">
            <span>• {item}</span>
            {!readOnly && (
              <button
                type="button"
                onClick={() => onChange?.(items.filter((_, j) => j !== i))}
                className="text-red-600 opacity-60 group-hover:opacity-100"
                aria-label={`Remove ${item}`}
              >
                ✕
              </button>
            )}
          </li>
        ))}
      </ul>
      {adding && !readOnly && (
        <form
          onSubmit={(e) => {
            e.preventDefault()
            add()
          }}
          className="flex gap-1"
        >
          <input
            autoFocus
            value={text}
            onChange={(e) => setText(e.target.value)}
            onBlur={add}
            list={listId}
            placeholder={placeholder}
            className="min-w-0 flex-1 rounded border bg-white px-2 py-1 text-sm"
          />
          {suggestions && (
            <datalist id={listId}>
              {suggestions.map((s) => (
                <option key={s} value={s} />
              ))}
            </datalist>
          )}
        </form>
      )}
    </div>
  )
}
