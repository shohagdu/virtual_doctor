import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { MedicineLine } from '../../api/types'
import { bnDigits } from '../../utils/money'
import { medicineName } from '../prescription/medicine'

const num = (s: string) => {
  const x = Number(s)
  return s.trim() !== '' && Number.isFinite(x) && x > 0 ? x : null
}
/** Rounded down to a measurable step (0.5 ml for liquids). */
const floorTo = (x: number, step: number) => Math.floor(x / step) * step
const fmt = (x: number) => String(Math.round(x * 100) / 100)

/**
 * 🧮 Weight-based dose (roadmap 2.4). It only does the arithmetic: the doctor enters
 * the mg/kg dose and the preparation's strength, and checks the result.
 */
export function DoseCalculator({
  weight,
  lines,
  onApply,
  onClose,
}: {
  weight: number | null
  lines: MedicineLine[]
  onApply: (index: number, notes: string) => void
  onClose: () => void
}) {
  const { t } = useTranslation()
  const named = lines.map((l, i) => ({ l, i })).filter(({ l }) => l.brand.trim())
  const [f, setF] = useState({
    weight: weight ? String(weight) : '',
    mgPerKg: '',
    perDay: '3',
    mg: '',
    ml: '5',
    max: '',
    line: named[0]?.i ?? -1,
  })
  const set = (patch: Partial<typeof f>) => setF({ ...f, ...patch })

  const w = num(f.weight)
  const mgPerKg = num(f.mgPerKg)
  const perDay = num(f.perDay)
  const mg = num(f.mg)
  const ml = num(f.ml)
  const max = num(f.max)
  const raw = w && mgPerKg ? w * mgPerKg : null
  const capped = raw !== null && max !== null && raw > max
  const dose = raw === null ? null : capped ? max! : raw
  const volume = dose !== null && mg && ml ? floorTo((dose * ml) / mg, 0.5) : null
  const text =
    dose !== null && perDay
      ? volume !== null
        ? `${bnDigits(fmt(volume))} মি.লি. করে দিনে ${bnDigits(fmt(perDay))} বার (${fmt(dose)} mg/dose)`
        : `${fmt(dose)} mg করে দিনে ${bnDigits(fmt(perDay))} বার`
      : ''
  const [edited, setEdited] = useState<string | null>(null)
  const notes = edited ?? text

  const input = 'mt-1 w-full rounded-md border px-2 py-1'
  return (
    <div className="overlay print:hidden">
      <div className="w-full max-w-md space-y-3 rounded-lg bg-white p-6 text-sm shadow-lg">
        <h2 className="text-lg font-semibold">🧮 {t('rxtools.doseCalc')}</h2>
        <div className="grid grid-cols-3 gap-2">
          <label>
            {t('rxtools.weight')} (kg)
            <input type="number" step="0.1" min="0" value={f.weight} onChange={(e) => set({ weight: e.target.value })} className={input} />
          </label>
          <label>
            mg/kg/{t('rxtools.dose')}
            <input type="number" step="any" min="0" value={f.mgPerKg} onChange={(e) => set({ mgPerKg: e.target.value })} className={input} />
          </label>
          <label>
            {t('rxtools.timesPerDay')}
            <input type="number" min="1" max="8" value={f.perDay} onChange={(e) => set({ perDay: e.target.value })} className={input} />
          </label>
          <label>
            {t('rxtools.strengthMg')}
            <input type="number" step="any" min="0" value={f.mg} onChange={(e) => set({ mg: e.target.value })} placeholder="120" className={input} />
          </label>
          <label>
            {t('rxtools.perMl')}
            <input type="number" step="any" min="0" value={f.ml} onChange={(e) => set({ ml: e.target.value })} className={input} />
          </label>
          <label>
            {t('rxtools.maxDose')}
            <input type="number" step="any" min="0" value={f.max} onChange={(e) => set({ max: e.target.value })} placeholder="mg" className={input} />
          </label>
        </div>

        {dose !== null && (
          <dl className="grid grid-cols-2 gap-y-1 rounded-md bg-panel p-3">
            <dt>{t('rxtools.perDose')}</dt>
            <dd className="text-right font-semibold">
              {fmt(dose)} mg{volume !== null && ` = ${fmt(volume)} ml`}
            </dd>
            {perDay && (
              <>
                <dt>{t('rxtools.perDayTotal')}</dt>
                <dd className="text-right">{fmt(dose * perDay)} mg</dd>
              </>
            )}
            {capped && <dd className="col-span-2 text-amber-700">⚠ {t('rxtools.capped', { raw: fmt(raw!) })}</dd>}
          </dl>
        )}

        {named.length > 0 && text && (
          <div className="space-y-1">
            <label className="block">
              {t('rxtools.applyTo')}
              <select value={f.line} onChange={(e) => set({ line: Number(e.target.value) })} className={`${input} bg-white`}>
                {named.map(({ l, i }) => (
                  <option key={i} value={i}>
                    {medicineName(l)}
                  </option>
                ))}
              </select>
            </label>
            <input value={notes} onChange={(e) => setEdited(e.target.value)} className={input} aria-label={t('rxtools.noteText')} />
          </div>
        )}
        <p className="text-xs text-slate-500">{t('rxtools.calcDisclaimer')}</p>
        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="btn btn-secondary">
            {t('common.close')}
          </button>
          <button
            onClick={() => {
              onApply(f.line, notes)
              onClose()
            }}
            disabled={!notes || f.line < 0}
            className="btn btn-primary"
          >
            {t('rxtools.addToLine')}
          </button>
        </div>
      </div>
    </div>
  )
}
