import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { Report, ResultInput } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { useSaveResults } from './api'
import { statusStyle } from './lab'

type Row = ResultInput & { key: number; status: Report['results'][number]['status']; source: 'lab' | 'system' | null }

function rowsOf(r: Report): Row[] {
  return r.results.map((x, i) => ({
    key: i,
    test_name: x.test_name,
    value_text: x.value_text,
    unit: x.unit,
    ref_range_text: x.ref_range_text,
    lab_flag: x.lab_flag,
    confidence: x.confidence,
    edited: x.edited_by_doctor,
    status: x.status,
    source: x.range_source,
  }))
}

const cell = 'w-full rounded border px-1.5 py-1 text-sm'

/**
 * Extracted values, for the doctor to check and correct before analysis (§6.3,
 * FR-13, FR-14). Status comes from the server's rule engine, so edited rows show it
 * again after saving.
 */
export function ExtractedValuesTable({ report, startEditing }: { report: Report; startEditing?: boolean }) {
  const { t } = useTranslation()
  const save = useSaveResults(report.id)
  const [editing, setEditing] = useState(startEditing ?? report.status !== 'verified')
  const [rows, setRows] = useState<Row[]>(() => rowsOf(report))
  const [nextKey, setNextKey] = useState(rows.length)
  const [head, setHead] = useState({
    report_type: report.report_type ?? '',
    lab_name: report.lab_name ?? '',
    report_date: report.report_date ?? '',
    impression: report.impression ?? '',
  })

  const setRow = (key: number, patch: Partial<Row>) =>
    setRows(rows.map((r) => (r.key === key ? { ...r, ...patch, edited: true, status: null, source: null } : r)))

  const submit = async () => {
    const saved = await save.mutateAsync({
      report_type: head.report_type || null,
      lab_name: head.lab_name || null,
      report_date: head.report_date || null,
      impression: head.impression || null,
      results: rows.map(({ key: _k, status: _s, source: _src, ...r }) => r),
    })
    setRows(rowsOf(saved))
    setEditing(false)
  }

  const critical = report.results.filter((r) => r.status === 'critical')

  return (
    <div className="space-y-2 text-sm">
      {critical.length > 0 && (
        <div className="rounded bg-red-600 px-3 py-2 font-semibold text-white" role="alert">
          ⚠ {t('reports.critical')}: {critical.map((c) => `${c.test_name} ${c.value_text} ${c.unit ?? ''}`.trim()).join(', ')}
        </div>
      )}
      {report.name_mismatch && (
        <p className="rounded bg-amber-50 px-3 py-2 text-amber-800">⚠ {t('reports.nameMismatch')}</p>
      )}
      {report.unreadable_parts.length > 0 && (
        <p className="text-xs text-amber-700">
          {t('reports.unreadable')}: {report.unreadable_parts.join('; ')}
        </p>
      )}

      {editing ? (
        <div className="grid grid-cols-3 gap-2">
          <input
            value={head.report_type}
            onChange={(e) => setHead({ ...head, report_type: e.target.value })}
            placeholder={t('reports.type')}
            aria-label={t('reports.type')}
            className={cell}
          />
          <input
            type="date"
            value={head.report_date}
            onChange={(e) => setHead({ ...head, report_date: e.target.value })}
            aria-label={t('reports.date')}
            className={cell}
          />
          <input
            value={head.lab_name}
            onChange={(e) => setHead({ ...head, lab_name: e.target.value })}
            placeholder={t('reports.lab')}
            aria-label={t('reports.lab')}
            className={cell}
          />
        </div>
      ) : (
        <p className="text-slate-600">
          <b>{report.report_type ?? t('reports.untitled')}</b>
          {report.report_date && <> · {fmtDate(report.report_date)}</>}
          {report.lab_name && <> · {report.lab_name}</>}
        </p>
      )}

      <div className="overflow-x-auto">
        <table className="w-full text-left">
          <thead className="border-b text-xs text-slate-500">
            <tr>
              <th className="py-1 pr-2">{t('reports.test')}</th>
              <th className="py-1 pr-2">{t('reports.result')}</th>
              <th className="py-1 pr-2">{t('reports.unit')}</th>
              <th className="py-1 pr-2">{t('reports.range')}</th>
              <th className="py-1 pr-2">{t('reports.status')}</th>
              {editing && <th />}
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => {
              const s = r.status ? statusStyle[r.status] : null
              // Once the doctor has checked the report, low-confidence reads are no longer in doubt.
              const unsure = report.status !== 'verified' && (r.confidence === 'low' || r.confidence === 'medium')
              return (
                <tr key={r.key} className={`border-b last:border-0 ${unsure && !r.edited ? 'bg-amber-50' : ''}`}>
                  {editing ? (
                    <>
                      <td className="py-1 pr-2">
                        <input value={r.test_name} onChange={(e) => setRow(r.key, { test_name: e.target.value })} className={cell} aria-label={t('reports.test')} />
                      </td>
                      <td className="py-1 pr-2">
                        <input value={r.value_text} onChange={(e) => setRow(r.key, { value_text: e.target.value })} className={`${cell} w-24`} aria-label={t('reports.result')} />
                      </td>
                      <td className="py-1 pr-2">
                        <input value={r.unit ?? ''} onChange={(e) => setRow(r.key, { unit: e.target.value || null })} className={`${cell} w-20`} aria-label={t('reports.unit')} />
                      </td>
                      <td className="py-1 pr-2">
                        <input value={r.ref_range_text ?? ''} onChange={(e) => setRow(r.key, { ref_range_text: e.target.value || null })} className={`${cell} w-28`} aria-label={t('reports.range')} />
                      </td>
                    </>
                  ) : (
                    <>
                      <td className="py-1 pr-2">{r.test_name}</td>
                      <td className={`py-1 pr-2 ${s?.cls ?? ''}`}>{r.value_text}</td>
                      <td className="py-1 pr-2 text-slate-500">{r.unit}</td>
                      <td className="py-1 pr-2 text-slate-500">
                        {r.ref_range_text}
                        {r.source === 'system' && <span title={t('reports.systemRange')}> ⓢ</span>}
                      </td>
                    </>
                  )}
                  <td className={`py-1 pr-2 whitespace-nowrap ${s?.cls ?? 'text-slate-400'}`}>
                    {s ? `${s.icon} ${t(`reports.statuses.${r.status}`)}` : '—'}
                    {unsure && !r.edited && <span className="ml-1 text-xs text-amber-700">⚠ {t('reports.verify')}</span>}
                  </td>
                  {editing && (
                    <td>
                      <button
                        type="button"
                        onClick={() => setRows(rows.filter((x) => x.key !== r.key))}
                        className="px-1 text-red-600"
                        aria-label={t('reports.removeRow')}
                      >
                        ✕
                      </button>
                    </td>
                  )}
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {editing && (
        <>
          <button
            type="button"
            onClick={() => {
              setRows([
                ...rows,
                { key: nextKey, test_name: '', value_text: '', unit: null, ref_range_text: null, lab_flag: null, confidence: null, edited: true, status: null, source: null },
              ])
              setNextKey(nextKey + 1)
            }}
            className="text-xs text-brand"
          >
            ➕ {t('reports.addRow')}
          </button>
          <textarea
            value={head.impression}
            onChange={(e) => setHead({ ...head, impression: e.target.value })}
            placeholder={t('reports.impression')}
            rows={2}
            className="w-full rounded border px-2 py-1"
          />
        </>
      )}
      {!editing && report.impression && (
        <p className="whitespace-pre-line">
          <b>{t('reports.impression')}:</b> {report.impression}
        </p>
      )}
      {save.isError && <p className="text-red-600">{save.error.message}</p>}
      <div className="flex gap-2">
        {editing ? (
          <button onClick={submit} disabled={save.isPending} className="rounded bg-brand px-3 py-1 text-white disabled:opacity-60">
            ✔ {t('reports.saveVerified')}
          </button>
        ) : (
          <button onClick={() => setEditing(true)} className="rounded border px-3 py-1">
            ✎ {t('reports.editValues')}
          </button>
        )}
        {report.status === 'verified' && !editing && <span className="self-center text-xs text-green-700">✔ {t('reports.verified')}</span>}
      </div>
    </div>
  )
}
