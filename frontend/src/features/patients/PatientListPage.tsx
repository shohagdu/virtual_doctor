import { useDeferredValue, useState } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import { format } from 'date-fns'
import { Card } from '../../components/Card'
import { ageText } from '../../utils/age'
import { usePatients } from './api'
import { useAiAvailable, useSmartSearch } from '../ai/api'
import { useAuth } from '../auth/authStore'
import type { SearchFilters } from '../../api/types'
import { fmtDate } from '../../utils/date'
import { PatientFormDialog } from './PatientFormDialog'

export function PatientListPage() {
  const { t } = useTranslation()
  const [q, setQ] = useState('')
  const [page, setPage] = useState(1)
  const [showNew, setShowNew] = useState(false)
  const query = useDeferredValue(q)
  const role = useAuth((s) => s.user?.role)
  // ✨ Smart search (PM spec §7.2): doctors describe the patients in a sentence.
  const canAi = useAiAvailable() && role === 'doctor'
  const [aiMode, setAiMode] = useState(false)
  const [aiQuery, setAiQuery] = useState('')
  const smart = useSmartSearch(aiMode ? aiQuery : '')
  const plain = usePatients(query, page, !aiMode)
  const data = aiMode ? (smart.data ? { items: smart.data.items, total: smart.data.items.length, per_page: 100 } : undefined) : plain.data
  const isLoading = aiMode ? smart.isFetching : plain.isLoading
  const lastPage = data ? Math.max(1, Math.ceil(data.total / data.per_page)) : 1

  return (
    <>
      <Card
        title={t('patients.title')}
        actions={
          <>
            {canAi && (
              <button
                onClick={() => {
                  setAiMode(!aiMode)
                  setAiQuery('')
                }}
                aria-pressed={aiMode}
                className={`rounded px-3 py-1.5 text-sm ${aiMode ? 'bg-indigo-600 text-white' : 'border text-indigo-700'}`}
              >
                ✨ {t('ai.smartSearch')}
              </button>
            )}
            {aiMode ? (
              <form
                onSubmit={(e) => {
                  e.preventDefault()
                  setAiQuery(q.trim())
                }}
                className="flex gap-1"
              >
                <input
                  value={q}
                  onChange={(e) => setQ(e.target.value)}
                  placeholder={t('ai.smartPlaceholder')}
                  className="w-72 rounded border border-indigo-300 px-3 py-1.5 text-sm"
                  maxLength={300}
                />
                <button className="rounded bg-indigo-600 px-3 py-1.5 text-sm text-white">{t('ai.search')}</button>
              </form>
            ) : (
              <input
                value={q}
                onChange={(e) => {
                  setQ(e.target.value)
                  setPage(1)
                }}
                placeholder={`🔍 ${t('patients.search')}`}
                className="w-56 rounded border px-3 py-1.5 text-sm"
              />
            )}
            <button onClick={() => setShowNew(true)} className="rounded bg-brand px-3 py-1.5 text-sm text-white">
              ➕ {t('patients.new')}
            </button>
          </>
        }
      >
        {aiMode && smart.data && <Understood filters={smart.data.filters} />}
        {aiMode && smart.isError && <p className="mb-2 text-sm text-red-600">{smart.error.message}</p>}
        {aiMode && !aiQuery && <p className="mb-2 text-sm text-slate-500">{t('ai.smartHelp')}</p>}
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="border-b text-slate-500">
              <tr>
                <th className="py-2 pr-3">{t('patients.id')}</th>
                <th className="py-2 pr-3">{t('patients.name')}</th>
                <th className="py-2 pr-3">{t('patients.ageSex')}</th>
                <th className="py-2 pr-3">{t('patients.phone')}</th>
                <th className="py-2 pr-3">{t('patients.registered')}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {isLoading && (
                <tr>
                  <td colSpan={6} className="py-4 text-slate-500">
                    {t('common.loading')}
                  </td>
                </tr>
              )}
              {data?.items.length === 0 && (
                <tr>
                  <td colSpan={6} className="py-4 text-slate-500">
                    {t('patients.empty')}
                  </td>
                </tr>
              )}
              {data?.items.map((p) => (
                <tr key={p.id} className="border-b last:border-0 hover:bg-slate-50">
                  <td className="py-2 pr-3 font-mono text-xs">{p.patient_code}</td>
                  <td className="py-2 pr-3">
                    {[p.title, p.name_en].filter(Boolean).join(' ')}
                    {p.name_bn && <div className="text-xs text-slate-500">{p.name_bn}</div>}
                  </td>
                  <td className="py-2 pr-3">
                    {ageText(p.dob)} / {p.sex}
                  </td>
                  <td className="py-2 pr-3">{p.phone}</td>
                  <td className="py-2 pr-3">{format(new Date(p.created_at), 'dd MMM yyyy')}</td>
                  <td className="py-2 text-right">
                    <Link to={`/patients/${p.id}`} className="text-brand">
                      ›
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className={`mt-3 flex items-center justify-end gap-2 text-sm ${aiMode ? 'hidden' : ''}`}>
          <button disabled={page <= 1} onClick={() => setPage(page - 1)} className="rounded border px-3 py-1 disabled:opacity-40">
            {t('common.prev')}
          </button>
          <span>
            {page} / {lastPage}
          </span>
          <button
            disabled={page >= lastPage}
            onClick={() => setPage(page + 1)}
            className="rounded border px-3 py-1 disabled:opacity-40"
          >
            {t('common.next')}
          </button>
        </div>
      </Card>
      {showNew && <PatientFormDialog onClose={() => setShowNew(false)} />}
    </>
  )
}

/** How the sentence was understood, so the doctor can judge the results. */
function Understood({ filters: f }: { filters: SearchFilters }) {
  const { t } = useTranslation()
  const parts: string[] = []
  if (f.sex) parts.push(f.sex === 'F' ? t('ai.f.female') : t('ai.f.male'))
  if (f.age_min != null || f.age_max != null) parts.push(t('ai.f.age', { min: f.age_min ?? 0, max: f.age_max ?? '…' }))
  if (f.diagnosis_terms?.length) parts.push(`${t('rx.diagnosis')}: ${f.diagnosis_terms.join(' / ')}`)
  if (f.complaint_terms?.length) parts.push(`${t('rx.complaints')}: ${f.complaint_terms.join(' / ')}`)
  if (f.medicine_terms?.length) parts.push(`${t('ai.f.medicine')}: ${f.medicine_terms.join(' / ')}`)
  if (f.chronic_terms?.length) parts.push(`${t('profile.chronic')}: ${f.chronic_terms.join(' / ')}`)
  if (f.visited_from || f.visited_to)
    parts.push(`${t('ai.f.visited')} ${f.visited_from ? fmtDate(f.visited_from) : '…'} – ${f.visited_to ? fmtDate(f.visited_to) : t('billing.today')}`)
  return (
    <p className="mb-2 flex flex-wrap gap-1 text-xs">
      <span className="text-slate-500">{t('ai.understood')}:</span>
      {parts.map((p) => (
        <span key={p} className="rounded bg-indigo-50 px-1.5 text-indigo-800">
          {p}
        </span>
      ))}
    </p>
  )
}
