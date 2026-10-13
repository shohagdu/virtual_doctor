import { useDeferredValue, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Download, FileDown, Pencil, Pill, Plus, Power, Search, Trash2, Upload } from 'lucide-react'
import { ApiError } from '../../api/client'
import type { Medicine } from '../../api/types'
import { Card } from '../../components/Card'
import { PageHeader } from '../../components/PageHeader'
import { useAuth } from '../auth/authStore'
import { useCatalogue, useImportCatalogue } from '../rxtools/api'
import { taka } from '../../utils/money'
import { useDeleteMedicine, useMedicines, useSetActive, type MedicineQuery } from './api'
import { MedicineDialog } from './MedicineDialog'

const TEMPLATE = 'brand,generic,strength,form,company,price\n'

/** Downloads the catalogue CSV (the request needs the login token, so no plain link). */
async function exportCsv() {
  const token = useAuth.getState().token
  const res = await fetch('/api/v1/medicines/export', { headers: token ? { Authorization: `Bearer ${token}` } : {} })
  if (!res.ok) throw new ApiError(res.status, res.statusText)
  const url = URL.createObjectURL(await res.blob())
  const a = document.createElement('a')
  a.href = url
  a.download = 'medicines.csv'
  a.click()
  URL.revokeObjectURL(url)
}

/**
 * Medicines: the clinic's catalogue used for ℞ suggestions and the allergy check
 * (generic names). Add, edit, deactivate, delete unused ones, import / export CSV.
 */
export function MedicinesPage() {
  const { t } = useTranslation()
  const [q, setQ] = useState('')
  const [filter, setFilter] = useState<Omit<MedicineQuery, 'q'>>({ form: '', company: '', status: '', page: 1 })
  const query = useDeferredValue(q.trim())
  const { data, isError, error, isFetching } = useMedicines({ ...filter, q: query })
  const size = useCatalogue()
  const imp = useImportCatalogue()
  const [editing, setEditing] = useState<Medicine | 'new' | null>(null)
  const [exportError, setExportError] = useState<string | null>(null)
  const lastPage = data ? Math.max(1, Math.ceil(data.total / data.per_page)) : 1
  const setF = (patch: Partial<typeof filter>) => setFilter({ ...filter, page: 1, ...patch })

  return (
    <div className="space-y-5">
      <PageHeader
        title={t('medicines.title')}
        subtitle={size.data ? t('medicines.subtitle', { own: size.data.clinic_rows, shared: size.data.shared_rows }) : undefined}
        actions={
          <>
            <div className="flex items-center gap-1 rounded-xl border border-slate-200 bg-white p-1 shadow-xs">
              <label className="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-slate-600 hover:bg-slate-100" title={t('rxtools.importCsv')}>
                <Upload className="h-4 w-4" />
                <span className="hidden sm:inline">{t('rxtools.importCsv')}</span>
                <input
                  type="file"
                  accept=".csv,text/csv"
                  className="hidden"
                  onChange={async (e) => {
                    const file = e.target.files?.[0]
                    e.target.value = ''
                    if (file) imp.mutate(await file.text())
                  }}
                />
              </label>
              <a
                href={`data:text/csv;charset=utf-8,${encodeURIComponent(TEMPLATE)}`}
                download="medicines-template.csv"
                className="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-slate-600 hover:bg-slate-100"
                title={t('rxtools.csvTemplate')}
              >
                <FileDown className="h-4 w-4" />
                <span className="hidden sm:inline">{t('medicines.template')}</span>
              </a>
              <button
                type="button"
                onClick={() => exportCsv().catch((e) => setExportError((e as Error).message))}
                className="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-slate-600 hover:bg-slate-100"
                title={t('medicines.export')}
              >
                <Download className="h-4 w-4" />
                <span className="hidden sm:inline">{t('medicines.export')}</span>
              </button>
            </div>
            <button onClick={() => setEditing('new')} className="btn btn-primary">
              <Plus className="h-4 w-4" /> {t('medicines.add')}
            </button>
          </>
        }
      />

      {(imp.isPending || imp.data || imp.isError || exportError) && (
        <div className="rounded-xl border border-slate-200 bg-white p-4 text-sm shadow-xs">
          {imp.isPending && <p className="text-slate-500">{t('rxtools.importing', { name: 'CSV' })}</p>}
          {imp.isError && <p className="text-red-600">{imp.error.message}</p>}
          {exportError && <p className="text-red-600">{exportError}</p>}
          {imp.data && (
            <>
              <p className="font-medium text-emerald-700">✔ {t('rxtools.imported', { added: imp.data.added, updated: imp.data.updated })}</p>
              {imp.data.problems.length > 0 && (
                <ul className="mt-1 text-xs text-amber-800">
                  {imp.data.problems.map((p) => (
                    <li key={`${p.line}-${p.message}`}>
                      {t('rxtools.line', { n: p.line })}: {p.message}
                    </li>
                  ))}
                  {imp.data.more_problems > 0 && <li>{t('rxtools.moreProblems', { count: imp.data.more_problems })}</li>}
                </ul>
              )}
            </>
          )}
        </div>
      )}

      <Card>
        <div className="mb-4 flex flex-wrap items-center gap-2">
          <div className="relative min-w-0 flex-1 sm:max-w-sm">
            <Search className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-slate-400" />
            <input
              value={q}
              onChange={(e) => {
                setQ(e.target.value)
                setF({})
              }}
              placeholder={t('medicines.search')}
              className="w-full rounded-lg border py-2 pr-3 pl-9 text-sm"
              aria-label={t('medicines.search')}
            />
          </div>
          <select
            value={filter.form}
            onChange={(e) => setF({ form: e.target.value })}
            className="rounded-lg border bg-white px-3 py-2 text-sm"
            aria-label={t('medicines.form')}
          >
            <option value="">{t('medicines.allForms')}</option>
            {data?.forms.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
          <select
            value={filter.company}
            onChange={(e) => setF({ company: e.target.value })}
            className="max-w-48 rounded-lg border bg-white px-3 py-2 text-sm"
            aria-label={t('medicines.company')}
          >
            <option value="">{t('medicines.allCompanies')}</option>
            {data?.companies.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
          <div className="flex overflow-hidden rounded-lg border bg-white text-sm" role="tablist">
            {(['', 'active', 'inactive'] as const).map((s) => (
              <button
                key={s || 'all'}
                role="tab"
                aria-selected={filter.status === s}
                onClick={() => setF({ status: s })}
                className={`px-3 py-2 ${filter.status === s ? 'bg-brand text-white' : 'text-slate-600 hover:bg-slate-50'}`}
              >
                {t(`medicines.status.${s || 'all'}`)}
              </button>
            ))}
          </div>
          {data && <span className="ml-auto text-sm text-slate-500">{t('medicines.count', { count: data.total })}</span>}
        </div>

        {isError ? (
          <p className="text-red-600">{error.message}</p>
        ) : !data ? (
          <p className="text-slate-500">{t('common.loading')}</p>
        ) : data.items.length === 0 ? (
          <div className="flex flex-col items-center gap-2 py-12 text-center">
            <span className="flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-400">
              <Pill className="h-7 w-7" />
            </span>
            <p className="font-medium text-slate-700">{query || filter.form || filter.company || filter.status ? t('medicines.noMatch') : t('medicines.empty')}</p>
            <p className="max-w-md text-sm text-slate-500">{t('medicines.emptyHint')}</p>
          </div>
        ) : (
          <div className={`overflow-x-auto ${isFetching ? 'opacity-70' : ''}`}>
            <table className="w-full text-left text-sm">
              <thead className="border-b">
                <tr>
                  <th className="py-2 pr-3">{t('medicines.brand')}</th>
                  <th className="py-2 pr-3">{t('medicines.generic')}</th>
                  <th className="py-2 pr-3">{t('medicines.strength')}</th>
                  <th className="py-2 pr-3">{t('medicines.company')}</th>
                  <th className="py-2 pr-3 text-right">{t('medicines.price')}</th>
                  <th className="py-2 pr-3 text-right">{t('medicines.used')}</th>
                  <th className="py-2 pr-3">{t('medicines.statusCol')}</th>
                  <th className="py-2" />
                </tr>
              </thead>
              <tbody>
                {data.items.map((m) => (
                  <Row key={m.id} m={m} onEdit={() => setEditing(m)} />
                ))}
              </tbody>
            </table>
          </div>
        )}

        {data && lastPage > 1 && (
          <div className="mt-4 flex items-center justify-end gap-2 text-sm">
            <button disabled={filter.page <= 1} onClick={() => setFilter({ ...filter, page: filter.page - 1 })} className="btn btn-secondary btn-sm">
              {t('common.prev')}
            </button>
            <span className="tabular-nums">
              {filter.page} / {lastPage}
            </span>
            <button disabled={filter.page >= lastPage} onClick={() => setFilter({ ...filter, page: filter.page + 1 })} className="btn btn-secondary btn-sm">
              {t('common.next')}
            </button>
          </div>
        )}
      </Card>

      {editing && (
        <MedicineDialog
          medicine={editing === 'new' ? undefined : editing}
          forms={data?.forms ?? []}
          companies={data?.companies ?? []}
          onClose={() => setEditing(null)}
        />
      )}
    </div>
  )
}

function Row({ m, onEdit }: { m: Medicine; onEdit: () => void }) {
  const { t } = useTranslation()
  const toggle = useSetActive()
  const del = useDeleteMedicine()
  const [confirming, setConfirming] = useState(false)
  const error = toggle.error ?? del.error
  const icon = 'rounded-lg p-1.5 text-slate-500 hover:bg-slate-100 disabled:opacity-30 disabled:hover:bg-transparent'

  return (
    <>
      <tr className={`border-b last:border-0 ${m.is_active ? '' : 'text-slate-400'}`}>
        <td className="py-2.5 pr-3">
          <span className={`font-medium ${m.is_active ? 'text-slate-900' : ''}`}>{m.brand}</span>
          {m.form && <span className="ml-1.5 rounded-md bg-slate-100 px-1.5 py-0.5 text-xs text-slate-600">{m.form}</span>}
          {m.shared && <span className="ml-1.5 rounded-md bg-violet-50 px-1.5 py-0.5 text-xs text-violet-700">{t('medicines.shared')}</span>}
        </td>
        <td className="py-2.5 pr-3">{m.generic}</td>
        <td className="py-2.5 pr-3 whitespace-nowrap">{m.strength ?? '—'}</td>
        <td className="py-2.5 pr-3">{m.company ?? '—'}</td>
        <td className="py-2.5 pr-3 text-right whitespace-nowrap tabular-nums">{m.price != null ? taka(m.price) : '—'}</td>
        <td className="py-2.5 pr-3 text-right tabular-nums">{m.used || '—'}</td>
        <td className="py-2.5 pr-3">
          <span
            className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium ring-1 ${
              m.is_active ? 'bg-emerald-50 text-emerald-700 ring-emerald-200' : 'bg-slate-100 text-slate-500 ring-slate-200'
            }`}
          >
            <span className={`h-1.5 w-1.5 rounded-full ${m.is_active ? 'bg-emerald-500' : 'bg-slate-400'}`} />
            {m.is_active ? t('medicines.active') : t('medicines.inactive')}
          </span>
        </td>
        <td className="py-2.5 whitespace-nowrap">
          <div className="flex items-center justify-end gap-0.5">
            <button onClick={onEdit} disabled={m.shared} className={icon} title={t('medicines.edit')} aria-label={t('medicines.edit')}>
              <Pencil className="h-4 w-4" />
            </button>
            <button
              onClick={() => toggle.mutate({ id: m.id, is_active: !m.is_active })}
              disabled={m.shared || toggle.isPending}
              className={`${icon} ${m.is_active ? '' : 'text-emerald-600'}`}
              title={m.is_active ? t('medicines.deactivate') : t('medicines.activate')}
              aria-label={m.is_active ? t('medicines.deactivate') : t('medicines.activate')}
            >
              <Power className="h-4 w-4" />
            </button>
            {confirming ? (
              <button
                onClick={() => del.mutate(m.id, { onSettled: () => setConfirming(false) })}
                disabled={del.isPending}
                className="rounded-lg bg-red-600 px-2 py-1 text-xs font-medium text-white hover:bg-red-700"
              >
                {t('medicines.confirmDelete')}
              </button>
            ) : (
              <button
                onClick={() => setConfirming(true)}
                disabled={m.shared || m.used > 0}
                className={`${icon} hover:text-red-600`}
                title={m.used > 0 ? t('medicines.usedCannotDelete') : t('medicines.delete')}
                aria-label={t('medicines.delete')}
              >
                <Trash2 className="h-4 w-4" />
              </button>
            )}
          </div>
        </td>
      </tr>
      {error && (
        <tr>
          <td colSpan={8} className="pb-2 text-xs text-red-600">
            {error.message}
          </td>
        </tr>
      )}
    </>
  )
}
