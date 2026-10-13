import { useEffect, useRef, useState, type ComponentType } from 'react'
import { Check, Plus, Search, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

/**
 * A section heading with ＋ and a list of items with ✕ — the building block of the
 * prescription screen's and the profile's left panel.
 */
export function ListSection({
  title,
  items,
  onChange,
  placeholder,
  readOnly = false,
  suggestions,
  icon: Icon,
  tone = 'default',
}: {
  title: string
  items: string[]
  onChange?: (items: string[]) => void
  placeholder?: string
  readOnly?: boolean
  suggestions?: string[]
  icon?: ComponentType<{ className?: string }>
  /** "danger" shows items as red tags (allergies); "chip" as neutral tags. */
  tone?: 'default' | 'danger' | 'chip'
}) {
  const [adding, setAdding] = useState(false)
  const remove = (i: number) => onChange?.(items.filter((_, j) => j !== i))

  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2 text-sm font-semibold text-slate-800">
        {Icon && (
          <span
            className={`flex h-6 w-6 items-center justify-center rounded-md ${
              tone === 'danger' ? 'bg-red-50 text-red-600' : 'bg-white text-slate-500 ring-1 ring-slate-200'
            }`}
          >
            <Icon className="h-3.5 w-3.5" />
          </span>
        )}
        <span className="flex-1">{title}</span>
        {!readOnly && (
          <button
            type="button"
            onClick={() => setAdding(!adding)}
            className="flex h-6 w-6 items-center justify-center rounded-md text-slate-400 hover:bg-white hover:text-brand hover:shadow-xs"
            aria-label={`Add ${title}`}
            title={`Add ${title}`}
          >
            <Plus className="h-4 w-4" />
          </button>
        )}
      </div>
      {items.length === 0 && !adding && <div className={`text-xs text-slate-400 ${Icon ? 'pl-8' : ''}`}>—</div>}
      {tone === 'default' ? (
        <ul className={`space-y-0.5 ${Icon ? 'pl-8' : ''}`}>
          {items.map((item, i) => (
            <li key={item} className="group flex items-start justify-between gap-2 rounded-md text-sm">
              <span className="flex gap-1.5">
                <span className="mt-2 h-1 w-1 shrink-0 rounded-full bg-slate-400" />
                {item}
              </span>
              {!readOnly && (
                <button
                  type="button"
                  onClick={() => remove(i)}
                  className="rounded p-0.5 text-slate-300 group-hover:text-red-500 hover:bg-red-50"
                  aria-label={`Remove ${item}`}
                >
                  <X className="h-3.5 w-3.5" />
                </button>
              )}
            </li>
          ))}
        </ul>
      ) : (
        <ul className={`flex flex-wrap gap-1.5 ${Icon ? 'pl-8' : ''}`}>
          {items.map((item, i) => (
            <li
              key={item}
              className={`inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium ring-1 ${
                tone === 'danger' ? 'bg-red-50 text-red-700 ring-red-200' : 'bg-white text-slate-700 ring-slate-200'
              }`}
            >
              {item}
              {!readOnly && (
                <button type="button" onClick={() => remove(i)} className="-mr-1 rounded-full p-0.5 opacity-60 hover:opacity-100" aria-label={`Remove ${item}`}>
                  <X className="h-3 w-3" />
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
      {adding && !readOnly && (
        <div className={Icon ? 'pl-8' : ''}>
          <PickInput
            options={suggestions ?? []}
            taken={items}
            placeholder={placeholder}
            onPick={(v) => {
              if (!items.includes(v)) onChange?.([...items, v])
            }}
            onDone={() => setAdding(false)}
          />
        </div>
      )}
    </div>
  )
}

/**
 * Type to filter the clinic's list (Settings → Prescription lists) and click or press
 * Enter to add; several items can be added in a row. Anything not in the list can be
 * typed and added too.
 */
function PickInput({
  options,
  taken,
  placeholder,
  onPick,
  onDone,
}: {
  options: string[]
  taken: string[]
  placeholder?: string
  onPick: (v: string) => void
  onDone: () => void
}) {
  const { t } = useTranslation()
  const [text, setText] = useState('')
  const [active, setActive] = useState(0)
  const box = useRef<HTMLDivElement>(null)
  const q = text.trim().toLowerCase()
  const matches = options.filter((o) => !taken.includes(o) && (!q || o.toLowerCase().includes(q))).slice(0, 60)
  const exact = options.some((o) => o.toLowerCase() === q) || taken.some((o) => o.toLowerCase() === q)
  const custom = q && !exact ? text.trim() : null
  // Rows: matching list items first (Enter takes the best match), then the typed text.
  const rows = custom ? [...matches, custom] : matches
  const index = Math.min(active, Math.max(rows.length - 1, 0))

  useEffect(() => {
    const close = (e: MouseEvent) => !box.current?.contains(e.target as Node) && onDone()
    document.addEventListener('mousedown', close)
    return () => document.removeEventListener('mousedown', close)
  }, [onDone])

  const pick = (v: string) => {
    onPick(v)
    setText('')
    setActive(0)
  }

  return (
    <div ref={box} className="relative">
      <div className="relative">
        <Search className="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-slate-400" />
        <input
          autoFocus
          value={text}
          onChange={(e) => {
            setText(e.target.value)
            setActive(0)
          }}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') {
              e.preventDefault()
              setActive(Math.min(index + 1, rows.length - 1))
            } else if (e.key === 'ArrowUp') {
              e.preventDefault()
              setActive(Math.max(index - 1, 0))
            } else if (e.key === 'Enter') {
              e.preventDefault()
              if (rows[index]) pick(rows[index])
            } else if (e.key === 'Escape') {
              onDone()
            }
          }}
          placeholder={placeholder ?? t('lists.searchOrType')}
          className="w-full rounded-lg border bg-white py-1.5 pr-2 pl-8 text-sm"
          role="combobox"
          aria-expanded={rows.length > 0}
          aria-autocomplete="list"
        />
      </div>
      <ul className="absolute z-30 mt-1 max-h-64 w-full overflow-y-auto rounded-lg border border-slate-200 bg-white py-1 text-sm shadow-lg" role="listbox">
        {rows.map((v, i) => (
          <li key={`${i}-${v}`} role="option" aria-selected={i === index}>
            <button
              type="button"
              onMouseDown={(e) => e.preventDefault()}
              onMouseEnter={() => setActive(i)}
              onClick={() => pick(v)}
              className={`flex w-full items-center gap-2 px-3 py-1.5 text-left ${i === index ? 'bg-brand-50 text-brand' : 'text-slate-700'}`}
            >
              {v === custom && i === rows.length - 1 ? (
                <>
                  <Plus className="h-3.5 w-3.5 shrink-0" />
                  <span className="truncate">
                    {t('lists.addTyped')} “<b>{v}</b>”
                  </span>
                </>
              ) : (
                <span className="truncate">{v}</span>
              )}
            </button>
          </li>
        ))}
        {rows.length === 0 && (
          <li className="px-3 py-2 text-xs text-slate-400">{options.length ? t('lists.allAdded') : t('lists.typeToAdd')}</li>
        )}
        <li className="flex items-center justify-between border-t border-slate-100 px-3 pt-1.5 pb-0.5 text-[11px] text-slate-400">
          <span>↑↓ · Enter</span>
          <button type="button" onMouseDown={(e) => e.preventDefault()} onClick={onDone} className="flex items-center gap-1 text-slate-500 hover:text-brand">
            <Check className="h-3 w-3" /> {t('lists.done')}
          </button>
        </li>
      </ul>
    </div>
  )
}
