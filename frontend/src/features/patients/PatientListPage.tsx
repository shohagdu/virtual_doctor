import { useDeferredValue, useState } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { differenceInCalendarDays, parseISO } from 'date-fns'
import {
  AlertTriangle,
  ArrowUpDown,
  CalendarPlus,
  ChevronLeft,
  ChevronRight,
  Phone,
  Search,
  Sparkles,
  UserPlus,
  Users,
  X,
} from 'lucide-react'
import type { Patient, PatientListItem, PatientSort, SearchFilters } from '../../api/types'
import { PageHeader } from '../../components/PageHeader'
import { ageText } from '../../utils/age'
import { fmtDate } from '../../utils/date'
import { fullName } from '../../utils/patient'
import { useAiAvailable, useSmartSearch } from '../ai/api'
import { useAuth } from '../auth/authStore'
import { usePatients } from './api'
import { PatientFormDialog } from './PatientFormDialog'

const PER_PAGE = 20
type Row = Patient & Partial<Pick<PatientListItem, 'visit_count' | 'last_visit' | 'has_allergy'>>

function initials(name: string) {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase())
    .join('')
}

const avatarTone = { M: 'bg-brand-100 text-brand', F: 'bg-pink-100 text-pink-700', O: 'bg-slate-200 text-slate-600' }

/** Patients: search, filter by sex, sort, and open a profile (§5.3). */
export function PatientListPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  // The top-bar search opens this page with ?q= (Enter in quick search).
  const [params] = useSearchParams()
  const [q, setQ] = useState(() => params.get('q') ?? '')
  const [page, setPage] = useState(1)
  const [sex, setSex] = useState('')
  const [sort, setSort] = useState<PatientSort>('newest')
  const [showNew, setShowNew] = useState(false)
  const query = useDeferredValue(q.trim())
  const role = useAuth((s) => s.user?.role)
  // ✨ Smart search (PM spec §7.2): doctors describe the patients in a sentence.
  const canAi = useAiAvailable() && role === 'doctor'
  const [aiMode, setAiMode] = useState(false)
  const [aiQuery, setAiQuery] = useState('')
  const smart = useSmartSearch(aiMode ? aiQuery : '')
  const plain = usePatients(query, page, !aiMode, { sex, sort, perPage: PER_PAGE })
  const all = usePatients('', 1, true, { perPage: 1 })

  const rows: Row[] | undefined = aiMode ? smart.data?.items : plain.data?.items
  const total = aiMode ? (smart.data?.items.length ?? 0) : (plain.data?.total ?? 0)
  const loading = aiMode ? smart.isFetching : plain.isLoading
  const lastPage = Math.max(1, Math.ceil(total / PER_PAGE))
  const filtered = !!(query || sex)
  const reset = (fn: () => void) => {
    fn()
    setPage(1)
  }

  return (
    <div className="space-y-5">
      <PageHeader
        title={t('patients.title')}
        subtitle={all.data ? t('patients.registeredCount', { count: all.data.total }) : undefined}
        actions={
          <>
            {canAi && (
              <button
                onClick={() => {
                  setAiMode(!aiMode)
                  setAiQuery('')
                }}
                aria-pressed={aiMode}
                className={`btn ${aiMode ? 'bg-indigo-600 text-white hover:bg-indigo-700' : 'btn-secondary text-indigo-700'}`}
              >
                <Sparkles className="h-4 w-4" /> {t('ai.smartSearch')}
              </button>
            )}
            <button onClick={() => setShowNew(true)} className="btn btn-primary">
              <UserPlus className="h-4 w-4" /> {t('patients.new')}
            </button>
          </>
        }
      />

      <section className="rounded-2xl border border-slate-200/80 bg-white shadow-sm">
        {/* Search and filters */}
        <div className="flex flex-wrap items-center gap-2 border-b border-slate-100 p-4">
          {aiMode ? (
            <form
              onSubmit={(e) => {
                e.preventDefault()
                setAiQuery(q.trim())
              }}
              className="flex min-w-0 flex-1 gap-2"
            >
              <div className="relative min-w-0 flex-1">
                <Sparkles className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-indigo-400" />
                <input
                  value={q}
                  onChange={(e) => setQ(e.target.value)}
                  placeholder={t('ai.smartPlaceholder')}
                  className="w-full rounded-lg border border-indigo-200 bg-indigo-50/40 py-2.5 pr-3 pl-9 text-sm"
                  maxLength={300}
                  autoFocus
                />
              </div>
              <button className="btn bg-indigo-600 text-white hover:bg-indigo-700">{t('ai.search')}</button>
            </form>
          ) : (
            <>
              <div className="relative min-w-0 flex-1 basis-full sm:basis-auto sm:max-w-md">
                <Search className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                  value={q}
                  onChange={(e) => reset(() => setQ(e.target.value))}
                  placeholder={t('patients.searchLong')}
                  aria-label={t('patients.search')}
                  className="w-full rounded-lg border py-2.5 pr-9 pl-9 text-sm"
                />
                {q && (
                  <button
                    onClick={() => reset(() => setQ(''))}
                    className="absolute top-1/2 right-2 -translate-y-1/2 rounded-md p-1 text-slate-400 hover:text-slate-700"
                    aria-label={t('patients.clearSearch')}
                  >
                    <X className="h-4 w-4" />
                  </button>
                )}
              </div>
              <div className="flex overflow-hidden rounded-lg border bg-white text-sm" role="tablist" aria-label={t('patients.sexFilter')}>
                {(['', 'M', 'F', 'O'] as const).map((s) => (
                  <button
                    key={s || 'all'}
                    role="tab"
                    aria-selected={sex === s}
                    onClick={() => reset(() => setSex(s))}
                    className={`px-3 py-2 ${sex === s ? 'bg-brand text-white' : 'text-slate-600 hover:bg-slate-50'}`}
                  >
                    {t(`patients.sex.${s || 'all'}`)}
                  </button>
                ))}
              </div>
              <label className="relative">
                <ArrowUpDown className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <select
                  value={sort}
                  onChange={(e) => reset(() => setSort(e.target.value as PatientSort))}
                  className="rounded-lg border bg-white py-2 pr-8 pl-9 text-sm"
                  aria-label={t('patients.sortBy')}
                >
                  <option value="newest">{t('patients.sort.newest')}</option>
                  <option value="last_visit">{t('patients.sort.last_visit')}</option>
                  <option value="name">{t('patients.sort.name')}</option>
                </select>
              </label>
            </>
          )}
        </div>

        {aiMode && smart.data && (
          <div className="border-b border-slate-100 px-4 py-2">
            <Understood filters={smart.data.filters} />
          </div>
        )}
        {aiMode && smart.isError && <p className="px-4 py-2 text-sm text-red-600">{smart.error.message}</p>}
        {aiMode && !aiQuery && <p className="px-4 py-3 text-sm text-slate-500">{t('ai.smartHelp')}</p>}

        {/* Results */}
        {loading && !rows ? (
          <ul className="divide-y divide-slate-100">
            {Array.from({ length: 5 }, (_, i) => (
              <li key={i} className="flex animate-pulse items-center gap-3 px-4 py-4">
                <span className="h-10 w-10 rounded-full bg-slate-100" />
                <span className="h-4 w-48 rounded bg-slate-100" />
                <span className="ml-auto h-4 w-24 rounded bg-slate-100" />
              </li>
            ))}
          </ul>
        ) : !rows || rows.length === 0 ? (
          (!aiMode || aiQuery) && (
            <div className="flex flex-col items-center gap-2 px-4 py-14 text-center">
              <span className="flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-400">
                <Users className="h-7 w-7" />
              </span>
              <p className="font-medium text-slate-700">{filtered || aiMode ? t('patients.noMatch') : t('patients.noneYet')}</p>
              {!aiMode && (
                <button onClick={() => setShowNew(true)} className="btn btn-primary btn-sm mt-1">
                  <UserPlus className="h-4 w-4" /> {t('patients.new')}
                </button>
              )}
            </div>
          )
        ) : (
          <>
            {/* Table on larger screens */}
            <div className={`hidden overflow-x-auto md:block ${plain.isFetching && !aiMode ? 'opacity-70' : ''}`}>
              <table className="w-full text-left text-sm">
                <thead className="border-b border-slate-100">
                  <tr>
                    <th className="py-3 pr-3 pl-4">{t('patients.name')}</th>
                    <th className="py-3 pr-3">{t('patients.id')}</th>
                    <th className="py-3 pr-3">{t('patients.ageSex')}</th>
                    <th className="py-3 pr-3">{t('patients.phone')}</th>
                    <th className="py-3 pr-3">{t('patients.lastVisit')}</th>
                    <th className="py-3 pr-3">{t('patients.registered')}</th>
                    <th className="py-3 pr-4" />
                  </tr>
                </thead>
                <tbody>
                  {rows.map((p) => (
                    <tr
                      key={p.id}
                      onClick={() => navigate(`/patients/${p.id}`)}
                      className="group cursor-pointer border-b border-slate-100 last:border-0"
                    >
                      <td className="py-3 pr-3 pl-4">
                        <div className="flex items-center gap-3">
                          <span className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-sm font-bold ${avatarTone[p.sex]}`}>
                            {initials(p.name_en)}
                          </span>
                          <div className="min-w-0">
                            <div className="flex items-center gap-1.5">
                              <Link
                                to={`/patients/${p.id}`}
                                onClick={(e) => e.stopPropagation()}
                                className="truncate font-semibold text-slate-900 group-hover:text-brand"
                              >
                                {fullName(p)}
                              </Link>
                              {p.has_allergy && (
                                <span
                                  className="inline-flex shrink-0 items-center gap-0.5 rounded-full bg-red-50 px-1.5 text-[11px] font-semibold text-red-700 ring-1 ring-red-200"
                                  title={t('profile.allergies')}
                                >
                                  <AlertTriangle className="h-3 w-3" /> {t('dashboard.allergy')}
                                </span>
                              )}
                            </div>
                            {p.name_bn && <div className="truncate text-xs text-slate-500">{p.name_bn}</div>}
                          </div>
                        </div>
                      </td>
                      <td className="py-3 pr-3">
                        <span className="rounded-md bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-700">{p.patient_code}</span>
                      </td>
                      <td className="py-3 pr-3 whitespace-nowrap">
                        {ageText(p.dob)} · {t(`patients.sex.${p.sex}`)}
                      </td>
                      <td className="py-3 pr-3 whitespace-nowrap">
                        <a
                          href={`tel:${p.phone}`}
                          onClick={(e) => e.stopPropagation()}
                          className="inline-flex items-center gap-1.5 text-slate-700 tabular-nums hover:text-brand"
                        >
                          <Phone className="h-3.5 w-3.5 text-slate-400" />
                          {p.phone}
                        </a>
                      </td>
                      <td className="py-3 pr-3 whitespace-nowrap">
                        <LastVisit p={p} />
                      </td>
                      <td className="py-3 pr-3 whitespace-nowrap text-slate-500">{fmtDate(p.created_at)}</td>
                      <td className="py-3 pr-4">
                        <div className="flex items-center justify-end gap-1">
                          <button
                            onClick={(e) => {
                              e.stopPropagation()
                              navigate(`/appointments?book=${p.id}`)
                            }}
                            className="rounded-lg p-1.5 text-slate-400 opacity-0 group-hover:opacity-100 hover:bg-slate-100 hover:text-brand focus:opacity-100"
                            title={t('appointments.book')}
                            aria-label={t('appointments.book')}
                          >
                            <CalendarPlus className="h-4 w-4" />
                          </button>
                          <ChevronRight className="h-4 w-4 text-slate-300 group-hover:text-brand" />
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Cards on phones */}
            <ul className="divide-y divide-slate-100 md:hidden">
              {rows.map((p) => (
                <li key={p.id}>
                  <Link to={`/patients/${p.id}`} className="flex items-center gap-3 px-4 py-3 active:bg-slate-50">
                    <span className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-sm font-bold ${avatarTone[p.sex]}`}>
                      {initials(p.name_en)}
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="flex items-center gap-1.5">
                        <span className="truncate font-semibold text-slate-900">{fullName(p)}</span>
                        {p.has_allergy && <AlertTriangle className="h-3.5 w-3.5 shrink-0 text-red-600" aria-label={t('profile.allergies')} />}
                      </span>
                      <span className="block truncate text-xs text-slate-500">
                        {p.patient_code} · {ageText(p.dob)} · {t(`patients.sex.${p.sex}`)} · {p.phone}
                      </span>
                      <span className="block text-xs">
                        <LastVisit p={p} />
                      </span>
                    </span>
                    <ChevronRight className="h-4 w-4 shrink-0 text-slate-300" />
                  </Link>
                </li>
              ))}
            </ul>
          </>
        )}

        {/* Pagination */}
        {!aiMode && rows && rows.length > 0 && (
          <div className="flex flex-wrap items-center gap-2 border-t border-slate-100 px-4 py-3 text-sm text-slate-500">
            <span>{t('patients.showing', { from: (page - 1) * PER_PAGE + 1, to: Math.min(page * PER_PAGE, total), total })}</span>
            <div className="ml-auto flex items-center gap-1">
              <button disabled={page <= 1} onClick={() => setPage(page - 1)} className="btn btn-secondary btn-sm" aria-label={t('common.prev')}>
                <ChevronLeft className="h-4 w-4" />
              </button>
              <span className="px-2 tabular-nums">
                {page} / {lastPage}
              </span>
              <button disabled={page >= lastPage} onClick={() => setPage(page + 1)} className="btn btn-secondary btn-sm" aria-label={t('common.next')}>
                <ChevronRight className="h-4 w-4" />
              </button>
            </div>
          </div>
        )}
      </section>

      {showNew && <PatientFormDialog onClose={() => setShowNew(false)} />}
    </div>
  )
}

/** Visits record a date only, so this counts calendar days: Today, Yesterday, 5 days ago. */
function daysAgo(date: string, t: (k: string, o?: Record<string, unknown>) => string) {
  const d = differenceInCalendarDays(new Date(), parseISO(date))
  if (d <= 0) return t('patients.today')
  if (d === 1) return t('patients.yesterday')
  if (d < 60) return t('patients.daysAgo', { count: d })
  return fmtDate(date)
}

/** "3 days ago" with the date, or a "New" tag for a patient never seen. */
function LastVisit({ p }: { p: Row }) {
  const { t } = useTranslation()
  if (p.visit_count === undefined) return <span className="text-slate-400">—</span>
  if (!p.last_visit) return <span className="rounded-full bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-700">{t('patients.noVisits')}</span>
  return (
    <span className="text-slate-700" title={fmtDate(p.last_visit)}>
      {daysAgo(p.last_visit, t)}
      <span className="ml-1.5 text-xs text-slate-400">· {t('patients.nVisits', { count: p.visit_count })}</span>
    </span>
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
        <span key={p} className="rounded-md bg-indigo-50 px-1.5 text-indigo-800">
          {p}
        </span>
      ))}
    </p>
  )
}
