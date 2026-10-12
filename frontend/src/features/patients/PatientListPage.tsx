import { useDeferredValue, useState } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import { format } from 'date-fns'
import { Card } from '../../components/Card'
import { ageText } from '../../utils/age'
import { usePatients } from './api'
import { PatientFormDialog } from './PatientFormDialog'

export function PatientListPage() {
  const { t } = useTranslation()
  const [q, setQ] = useState('')
  const [page, setPage] = useState(1)
  const [showNew, setShowNew] = useState(false)
  const query = useDeferredValue(q)
  const { data, isLoading } = usePatients(query, page)
  const lastPage = data ? Math.max(1, Math.ceil(data.total / data.per_page)) : 1

  return (
    <>
      <Card
        title={t('patients.title')}
        actions={
          <>
            <input
              value={q}
              onChange={(e) => {
                setQ(e.target.value)
                setPage(1)
              }}
              placeholder={`🔍 ${t('patients.search')}`}
              className="w-56 rounded border px-3 py-1.5 text-sm"
            />
            <button onClick={() => setShowNew(true)} className="rounded bg-brand px-3 py-1.5 text-sm text-white">
              ➕ {t('patients.new')}
            </button>
          </>
        }
      >
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
        <div className="mt-3 flex items-center justify-end gap-2 text-sm">
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
