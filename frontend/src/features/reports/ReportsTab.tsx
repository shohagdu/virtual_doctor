import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { fileUrl } from '../../api/stream'
import type { Report } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { useLabTrends, usePatientReports } from './api'
import { statusStyle } from './lab'

/** Profile → Reports (FR-38, FR-39): every report by date, and a trend for repeated tests. */
export function ReportsTab({ patientId, clinical }: { patientId: string; clinical: boolean }) {
  const { t } = useTranslation()
  const { data, isLoading } = usePatientReports(patientId)
  if (isLoading || !data) return <p className="text-slate-500">{t('common.loading')}</p>
  if (data.length === 0) return <p className="text-slate-500">{t('reports.none')}</p>
  return (
    <div className="space-y-4">
      {clinical && <LabTrend patientId={patientId} />}
      <ul className="divide-y">
        {data.map((r) => (
          <ReportRow key={r.id} r={r} />
        ))}
      </ul>
    </div>
  )
}

function ReportRow({ r }: { r: Report }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const flagged = r.results.filter((x) => x.status && x.status !== 'normal')
  return (
    <li className="py-2 text-sm">
      <div className="flex flex-wrap items-center gap-2">
        <span className="w-24 whitespace-nowrap text-slate-500">{fmtDate(r.report_date ?? r.created_at)}</span>
        <b>{r.report_type ?? r.original_name ?? t('reports.untitled')}</b>
        {r.lab_name && <span className="text-xs text-slate-500">{r.lab_name}</span>}
        {flagged.length > 0 && (
          <span className="text-xs text-orange-700">
            {flagged.length} {t('reports.flagged')}
          </span>
        )}
        <span className="ml-auto flex gap-1">
          {r.results.length > 0 && (
            <button onClick={() => setOpen(!open)} className="rounded border px-2 py-0.5 text-xs">
              {open ? '▲' : '▼'} {t('reports.values')}
            </button>
          )}
          <button
            onClick={async () => window.open(await fileUrl(`/reports/${r.id}/file`), '_blank', 'noopener')}
            className="rounded border px-2 py-0.5 text-xs"
          >
            {t('reports.viewFile')} ↗
          </button>
        </span>
      </div>
      {open && (
        <table className="mt-2 w-full max-w-xl text-left text-sm">
          <tbody>
            {r.results.map((x) => {
              const s = x.status ? statusStyle[x.status] : null
              return (
                <tr key={x.id} className="border-b last:border-0">
                  <td className="py-0.5 pr-2">{x.test_name}</td>
                  <td className={`py-0.5 pr-2 ${s?.cls ?? ''}`}>
                    {x.value_text} {x.unit}
                  </td>
                  <td className="py-0.5 pr-2 text-slate-500">{x.ref_range_text}</td>
                  <td className={`py-0.5 ${s?.cls ?? ''}`}>{s && `${s.icon} ${t(`reports.statuses.${x.status}`)}`}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      )}
      {open && r.impression && <p className="mt-1 whitespace-pre-line text-slate-700">{r.impression}</p>}
    </li>
  )
}

/** One test over time (FR-39); a single series, so its name is the chart's title. */
function LabTrend({ patientId }: { patientId: string }) {
  const { t } = useTranslation()
  const [test, setTest] = useState<string | null>(null)
  const { data } = useLabTrends(patientId, test)
  if (!data || data.tests.length === 0) return null
  const repeated = data.tests.filter((x) => x.count > 1)
  const current = data.tests.find((x) => x.test_code === data.test)
  const unit = data.points.find((p) => p.unit)?.unit ?? ''
  const points = data.points.map((p) => ({ ...p, label: fmtDate(p.date) }))

  return (
    <section className="space-y-2">
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <h3 className="font-semibold">📈 {t('reports.trend')}</h3>
        <select
          value={data.test ?? ''}
          onChange={(e) => setTest(e.target.value)}
          className="rounded border bg-white px-2 py-1"
          aria-label={t('reports.test')}
        >
          {(repeated.length ? repeated : data.tests).map((x) => (
            <option key={x.test_code} value={x.test_code}>
              {x.test_name} ({x.count})
            </option>
          ))}
        </select>
      </div>
      {points.length < 2 ? (
        <p className="text-sm text-slate-500">{t('reports.oneValue', { name: current?.test_name ?? '' })}</p>
      ) : (
        <div className="h-56" aria-label={`${current?.test_name ?? ''} ${unit}`}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={points} margin={{ top: 8, right: 16, bottom: 0, left: 0 }}>
              <CartesianGrid stroke="#e2e8f0" vertical={false} />
              <XAxis dataKey="label" tick={{ fontSize: 12, fill: '#64748b' }} stroke="#cbd5e1" />
              <YAxis
                domain={['auto', 'auto']}
                tick={{ fontSize: 12, fill: '#64748b' }}
                stroke="#cbd5e1"
                width={56}
                label={{ value: unit, angle: -90, position: 'insideLeft', fontSize: 12, fill: '#64748b' }}
              />
              <Tooltip formatter={(v) => [`${v} ${unit}`, current?.test_name ?? '']} />
              <Line
                type="monotone"
                dataKey="value"
                stroke="#1d6fb8"
                strokeWidth={2}
                dot={{ r: 4, strokeWidth: 2, stroke: '#fff', fill: '#1d6fb8' }}
                activeDot={{ r: 6 }}
                isAnimationActive={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}
    </section>
  )
}
