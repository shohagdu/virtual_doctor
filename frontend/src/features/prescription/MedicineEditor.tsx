import { useDeferredValue, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { MedicineLine } from '../../api/types'
import { emptyLine } from './medicine'
import { useMedicineSearch } from './api'


const DOSES = ['1+0+1', '1+1+1', '1+0+0', '0+0+1', '0+1+0', '1+1+1+1', '½+0+½']
const TIMINGS = ['খাবারের পরে', 'খাবারের আগে', 'খাবারের সাথে', 'ঘুমানোর আগে', 'খালি পেটে']
const DURATIONS = ['৩ দিন', '৫ দিন', '৭ দিন', '১০ দিন', '১৪ দিন', '১ মাস', '৩ মাস', 'চলবে']

const cell = 'w-full rounded border bg-white px-2 py-1 text-sm'

/** The ℞ panel: one editable row per medicine. */
export function MedicineEditor({
  lines,
  onChange,
  flagged,
  readOnly,
  onFavorite,
  isFavorite,
}: {
  lines: MedicineLine[]
  onChange: (lines: MedicineLine[]) => void
  flagged: Set<string>
  readOnly: boolean
  /** Saves the line (brand, strength and dose) as a favourite (roadmap 2.3). */
  onFavorite?: (line: MedicineLine) => void
  isFavorite?: (line: MedicineLine) => boolean
}) {
  const { t } = useTranslation()
  const set = (i: number, patch: Partial<MedicineLine>) =>
    onChange(lines.map((l, j) => (j === i ? { ...l, ...patch } : l)))

  return (
    <div className="space-y-2">
      <datalist id="rx-doses">{DOSES.map((d) => <option key={d} value={d} />)}</datalist>
      <datalist id="rx-timings">{TIMINGS.map((d) => <option key={d} value={d} />)}</datalist>
      <datalist id="rx-durations">{DURATIONS.map((d) => <option key={d} value={d} />)}</datalist>

      {lines.map((line, i) => (
        <div
          key={i}
          className={`grid gap-2 rounded border p-2 sm:grid-cols-[2fr_1fr_1fr_1.3fr_1fr_auto] ${
            flagged.has(line.brand) ? 'border-red-500 bg-red-50' : 'border-slate-200'
          }`}
        >
          <BrandInput line={line} readOnly={readOnly} onPick={(patch) => set(i, patch)} />
          <input
            value={line.strength ?? ''}
            onChange={(e) => set(i, { strength: e.target.value || null })}
            placeholder={t('rx.strength')}
            readOnly={readOnly}
            className={cell}
          />
          <input
            value={line.dose_pattern ?? ''}
            onChange={(e) => set(i, { dose_pattern: e.target.value || null })}
            list="rx-doses"
            placeholder="1+0+1"
            readOnly={readOnly}
            className={cell}
          />
          <input
            value={line.timing ?? ''}
            onChange={(e) => set(i, { timing: e.target.value || null })}
            list="rx-timings"
            placeholder={t('rx.timing')}
            readOnly={readOnly}
            className={cell}
          />
          <input
            value={line.duration ?? ''}
            onChange={(e) => set(i, { duration: e.target.value || null })}
            list="rx-durations"
            placeholder={t('rx.duration')}
            readOnly={readOnly}
            className={cell}
          />
          {!readOnly && (
            <div className="flex items-start">
              {onFavorite && line.brand.trim() && (
                <button
                  type="button"
                  onClick={() => onFavorite(line)}
                  className={`px-1.5 ${isFavorite?.(line) ? 'text-amber-500' : 'text-slate-400 hover:text-amber-500'}`}
                  aria-label={t('rxtools.saveFavorite')}
                  title={t('rxtools.saveFavorite')}
                >
                  {isFavorite?.(line) ? '★' : '☆'}
                </button>
              )}
              {line.notes === null && line.brand.trim() && (
                <button
                  type="button"
                  onClick={() => set(i, { notes: '' })}
                  className="px-1.5 text-slate-400 hover:text-brand"
                  aria-label={t('rxtools.addNote')}
                  title={t('rxtools.addNote')}
                >
                  ✎
                </button>
              )}
              <button
                type="button"
                onClick={() => onChange(lines.filter((_, j) => j !== i))}
                className="px-2 text-red-600"
                aria-label={t('rx.removeMedicine')}
              >
                ✕
              </button>
            </div>
          )}
          {line.notes !== null && (
            <input
              value={line.notes}
              onChange={(e) => set(i, { notes: e.target.value })}
              onBlur={(e) => !e.target.value.trim() && set(i, { notes: null })}
              placeholder={t('rxtools.notePlaceholder')}
              readOnly={readOnly}
              aria-label={t('rxtools.addNote')}
              className={`${cell} sm:col-span-6`}
            />
          )}
        </div>
      ))}
      {!readOnly && (
        <button type="button" onClick={() => onChange([...lines, emptyLine])} className="text-sm text-brand">
          ➕ {t('rx.addMedicine')}
        </button>
      )}
    </div>
  )
}

function BrandInput({
  line,
  readOnly,
  onPick,
}: {
  line: MedicineLine
  readOnly: boolean
  onPick: (patch: Partial<MedicineLine>) => void
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const query = useDeferredValue(line.brand)
  const { data } = useMedicineSearch(open ? query : '')

  return (
    <div className="relative">
      <input
        value={line.brand}
        onChange={(e) => {
          onPick({ brand: e.target.value, medicine_id: null })
          setOpen(true)
        }}
        onFocus={() => setOpen(true)}
        onBlur={() => setTimeout(() => setOpen(false), 150)}
        placeholder={t('rx.brand')}
        readOnly={readOnly}
        className={`${cell} font-medium`}
      />
      {line.generic && <div className="px-1 text-xs text-slate-500">{line.generic}</div>}
      {open && !readOnly && data && data.length > 0 && (
        <ul className="absolute z-20 mt-1 max-h-60 w-72 overflow-y-auto rounded border bg-white shadow-lg">
          {data.map((m) => (
            <li key={`${m.brand}-${m.strength}`}>
              <button
                type="button"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  onPick({
                    medicine_id: m.medicine_id,
                    brand: m.brand,
                    generic: m.generic,
                    strength: m.strength ?? line.strength,
                    form: m.form,
                  })
                  setOpen(false)
                }}
                className="block w-full px-3 py-1.5 text-left text-sm hover:bg-slate-100"
              >
                <b>{[m.form, m.brand].filter(Boolean).join(' ')}</b> {m.strength}
                {m.generic && <span className="block text-xs text-slate-500">{m.generic}</span>}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
