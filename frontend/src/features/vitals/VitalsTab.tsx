import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CartesianGrid, Legend, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import type { VitalsRecord } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { useVitalsTrend } from './api'

type Key = 'bp_sys' | 'bp_dia' | 'pulse' | 'temp' | 'weight' | 'bmi' | 'spo2' | 'rbs'

// One measure per chart (never two y-scales); BP plots systolic and diastolic together.
const METRICS: { id: string; label: string; unit: string; series: { key: Key; name: string }[] }[] = [
  { id: 'bp', label: 'BP', unit: 'mmHg', series: [{ key: 'bp_sys', name: 'Systolic' }, { key: 'bp_dia', name: 'Diastolic' }] },
  { id: 'pulse', label: 'Pulse', unit: '/min', series: [{ key: 'pulse', name: 'Pulse' }] },
  { id: 'temp', label: 'Temp', unit: '°F', series: [{ key: 'temp', name: 'Temp' }] },
  { id: 'weight', label: 'Weight', unit: 'kg', series: [{ key: 'weight', name: 'Weight' }] },
  { id: 'bmi', label: 'BMI', unit: 'kg/m²', series: [{ key: 'bmi', name: 'BMI' }] },
  { id: 'spo2', label: 'SpO₂', unit: '%', series: [{ key: 'spo2', name: 'SpO₂' }] },
  { id: 'rbs', label: 'RBS', unit: 'mmol/L', series: [{ key: 'rbs', name: 'RBS' }] },
]
const COLORS = ['#1d6fb8', '#c2410c']

/** Vitals tab of the patient profile: trend chart + table (PM-13, §5.4). */
export function VitalsTab({ patientId }: { patientId: string }) {
  const { t } = useTranslation()
  const { data, isLoading, isError, error } = useVitalsTrend(patientId)
  const [metricId, setMetricId] = useState('bp')

  if (isError) return <p className="text-red-600">{error.message}</p>
  if (isLoading || !data) return <p className="text-slate-500">{t('common.loading')}</p>
  if (data.length === 0) return <p className="text-slate-500">{t('vitals.none')}</p>

  const metric = METRICS.find((m) => m.id === metricId) ?? METRICS[0]
  const points = data
    .filter((r) => metric.series.some((s) => r[s.key] !== null))
    .map((r) => ({ ...r, label: fmtDate(r.visit_date) }))

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-1" role="tablist">
        {METRICS.map((m) => (
          <button
            key={m.id}
            role="tab"
            aria-selected={m.id === metric.id}
            onClick={() => setMetricId(m.id)}
            className={`rounded-md px-3 py-1 text-sm ${m.id === metric.id ? 'bg-brand text-white' : 'border text-slate-600'}`}
          >
            {m.label}
          </button>
        ))}
      </div>

      {points.length === 0 ? (
        <p className="text-sm text-slate-500">{t('vitals.noneFor', { name: metric.label })}</p>
      ) : (
        <div className="h-64" aria-label={`${metric.label} (${metric.unit})`}>
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={points} margin={{ top: 8, right: 16, bottom: 0, left: 0 }}>
              <CartesianGrid stroke="#e2e8f0" vertical={false} />
              <XAxis dataKey="label" tick={{ fontSize: 12, fill: '#64748b' }} stroke="#cbd5e1" />
              <YAxis
                domain={['auto', 'auto']}
                tick={{ fontSize: 12, fill: '#64748b' }}
                stroke="#cbd5e1"
                width={48}
                label={{ value: metric.unit, angle: -90, position: 'insideLeft', fontSize: 12, fill: '#64748b' }}
              />
              <Tooltip formatter={(v, name) => [`${v} ${metric.unit}`, name]} />
              {metric.series.length > 1 && <Legend wrapperStyle={{ fontSize: 12 }} />}
              {metric.series.map((s, i) => (
                <Line
                  key={s.key}
                  type="monotone"
                  dataKey={s.key}
                  name={s.name}
                  stroke={COLORS[i]}
                  strokeWidth={2}
                  dot={{ r: 4, strokeWidth: 2, stroke: '#fff', fill: COLORS[i] }}
                  activeDot={{ r: 6 }}
                  connectNulls
                  isAnimationActive={false}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      <VitalsTable rows={data} />
    </div>
  )
}

function VitalsTable({ rows }: { rows: VitalsRecord[] }) {
  const { t } = useTranslation()
  const cell = (v: number | null) => (v === null ? '—' : v)
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead className="border-b text-slate-500">
          <tr>
            <th className="py-2 pr-3">{t('vitals.date')}</th>
            <th className="py-2 pr-3">BP</th>
            <th className="py-2 pr-3">Pulse</th>
            <th className="py-2 pr-3">Temp °F</th>
            <th className="py-2 pr-3">Wt kg</th>
            <th className="py-2 pr-3">Ht cm</th>
            <th className="py-2 pr-3">BMI</th>
            <th className="py-2 pr-3">SpO₂ %</th>
            <th className="py-2 pr-3">RBS</th>
          </tr>
        </thead>
        <tbody>
          {[...rows].reverse().map((r) => (
            <tr key={r.visit_id} className="border-b last:border-0">
              <td className="py-2 pr-3 whitespace-nowrap">{fmtDate(r.visit_date)}</td>
              <td className="py-2 pr-3">{r.bp_sys !== null ? `${r.bp_sys}/${r.bp_dia}` : '—'}</td>
              <td className="py-2 pr-3">{cell(r.pulse)}</td>
              <td className="py-2 pr-3">{cell(r.temp)}</td>
              <td className="py-2 pr-3">{cell(r.weight)}</td>
              <td className="py-2 pr-3">{cell(r.height)}</td>
              <td className="py-2 pr-3">{cell(r.bmi)}</td>
              <td className="py-2 pr-3">{cell(r.spo2)}</td>
              <td className="py-2 pr-3">{cell(r.rbs)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
