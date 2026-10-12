import { useState } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { MedicineLine, VisitSummary } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { medicineDirections, medicineName } from '../prescription/medicine'

const key = (m: MedicineLine) => m.brand.trim().toLowerCase()

/** Profile → Prescriptions (§5.4): every prescription, and any two side by side. */
export function PrescriptionsTab({ visits }: { visits: VisitSummary[] }) {
  const { t } = useTranslation()
  const withRx = visits.filter((v) => v.prescription)
  const [picked, setPicked] = useState<number[]>([])
  if (withRx.length === 0) return <p className="text-slate-500">{t('rxtools.noPrescriptions')}</p>
  const toggle = (id: number) =>
    setPicked(picked.includes(id) ? picked.filter((x) => x !== id) : [...picked, id].slice(-2))
  // Older on the left, newer on the right.
  const pair = withRx.filter((v) => picked.includes(v.id)).sort((a, b) => a.visit_date.localeCompare(b.visit_date) || a.id - b.id)

  return (
    <div className="space-y-4 text-sm">
      <p className="text-xs text-slate-500">{t('rxtools.compareHint')}</p>
      <ul className="divide-y">
        {withRx.map((v) => {
          const rx = v.prescription!
          return (
            <li key={v.id} className="flex items-start gap-2 py-2">
              <input
                type="checkbox"
                checked={picked.includes(v.id)}
                onChange={() => toggle(v.id)}
                className="mt-1"
                aria-label={t('rxtools.compare')}
              />
              <div className="min-w-0 flex-1">
                <b>{fmtDate(v.visit_date)}</b>
                {rx.diagnosis.length > 0 && <> — {rx.diagnosis.join(', ')}</>}
                <span className="ml-2 text-xs text-slate-500">
                  v{rx.version}
                  {rx.locked ? ' 🔒' : ''} · {t('rxtools.nMedicines', { count: rx.medicines.length })}
                </span>
                <div className="text-xs text-slate-500">{rx.medicines.map(medicineName).join(' · ')}</div>
              </div>
              <Link to={`/visits/${v.id}/prescription`} className="rounded border px-2 py-0.5 text-xs text-brand">
                {t('visits.viewRx')}
              </Link>
            </li>
          )
        })}
      </ul>
      {pair.length === 2 && <Compare a={pair[0]} b={pair[1]} />}
    </div>
  )
}

function Compare({ a, b }: { a: VisitSummary; b: VisitSummary }) {
  const { t } = useTranslation()
  const ra = a.prescription!
  const rb = b.prescription!
  const status = (m: MedicineLine, other: MedicineLine[]) => {
    const twin = other.find((o) => key(o) === key(m))
    if (!twin) return 'only'
    return medicineName(twin) === medicineName(m) && medicineDirections(twin) === medicineDirections(m) ? 'same' : 'changed'
  }
  const cls = { only: '', same: 'text-slate-500', changed: 'bg-amber-50' } as const
  const column = (v: VisitSummary, mine: MedicineLine[], other: MedicineLine[], onlyClass: string) => (
    <div className="space-y-2">
      <div className="font-semibold">{fmtDate(v.visit_date)}</div>
      <div>
        <span className="text-slate-500">{t('rx.diagnosis')}:</span> {v.prescription!.diagnosis.join(', ') || '—'}
      </div>
      <ul className="space-y-1">
        {mine.map((m, i) => {
          const s = status(m, other)
          return (
            <li key={i} className={`rounded px-1 ${s === 'only' ? onlyClass : cls[s]}`}>
              ℞ <b>{medicineName(m)}</b> {medicineDirections(m)}
            </li>
          )
        })}
      </ul>
    </div>
  )
  return (
    <section className="space-y-2 rounded border p-3">
      <h3 className="font-semibold">{t('rxtools.comparison')}</h3>
      <p className="flex flex-wrap gap-3 text-xs">
        <span className="rounded bg-red-50 px-1 text-red-800 line-through">{t('rxtools.stopped')}</span>
        <span className="rounded bg-green-50 px-1 text-green-800">{t('rxtools.started')}</span>
        <span className="rounded bg-amber-50 px-1">{t('rxtools.changed')}</span>
      </p>
      <div className="grid gap-4 md:grid-cols-2">
        {column(a, ra.medicines, rb.medicines, 'bg-red-50 text-red-800 line-through')}
        {column(b, rb.medicines, ra.medicines, 'bg-green-50 text-green-800')}
      </div>
    </section>
  )
}
