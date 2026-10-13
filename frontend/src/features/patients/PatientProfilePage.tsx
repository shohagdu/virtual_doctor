import { useState, type ComponentType } from 'react'
import { Link, useNavigate, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { VisitSummary } from '../../api/types'
import {
  Activity,
  BellRing,
  CalendarCheck,
  CalendarDays,
  CalendarPlus,
  Check,
  ChevronRight,
  Clock,
  FilePlus2,
  FileText,
  FlaskConical,
  ListPlus,
  Merge,
  Pencil,
  Sparkles,
  Wallet,
} from 'lucide-react'
import { Card } from '../../components/Card'
import { PatientInfoBar } from '../../components/PatientInfoBar'
import { todayIso } from '../../utils/date'
import { useAuth } from '../auth/authStore'
import { VitalsTab } from '../vitals/VitalsTab'
import { useAiAvailable } from '../ai/api'
import { AiSummary } from '../ai/AiSummary'
import { usePatientReports } from '../reports/api'
import { ReportsTab } from '../reports/ReportsTab'
import { BillingTab } from './BillingTab'
import { MergeDialog } from './MergeDialog'
import { PrescriptionsTab } from './PrescriptionsTab'
import { usePatientAppointments } from '../appointments/api'
import { fmtDate } from '../../utils/date'
import { useMedicalProfile, usePatient, useStartVisit, useVisits } from './api'
import { MedicalProfilePanel } from './MedicalProfilePanel'
import { PatientFormDialog } from './PatientFormDialog'
import { VisitTimeline } from './VisitTimeline'

type Tab = 'visits' | 'prescriptions' | 'vitals' | 'reports' | 'ai' | 'billing'

export function PatientProfilePage() {
  const { id = '' } = useParams()
  const { t } = useTranslation()
  const navigate = useNavigate()
  const role = useAuth((s) => s.user?.role)
  const clinicalAccess = role === 'doctor' || role === 'assistant'
  const { data: p, isLoading } = usePatient(id)
  const profile = useMedicalProfile(id, clinicalAccess)
  const visits = useVisits(id, clinicalAccess)
  const startVisit = useStartVisit(id)
  const reports = usePatientReports(id)
  const appointments = usePatientAppointments(id)
  const upcoming = appointments.data?.filter((a) => a.status === 'booked') ?? []
  const [editing, setEditing] = useState(false)
  const [tab, setTab] = useState<Tab>('visits')
  const aiAvailable = useAiAvailable()
  const tabs: Tab[] =
    role === 'doctor' ? ['visits', 'prescriptions', 'vitals', 'reports', 'ai', 'billing'] : ['visits', 'vitals', 'reports', 'billing']
  const [merging, setMerging] = useState(false)

  if (isLoading || !p) return <p className="text-slate-500">{t('common.loading')}</p>

  // Reuse today's open visit (e.g. one the assistant queued) instead of starting another.
  const openToday = visits.data?.find(
    (v) => v.visit_date === todayIso() && (v.status === 'waiting' || v.status === 'in_consultation'),
  )
  const todayVisit = async () => openToday ?? (await startVisit.mutateAsync(visits.data?.length ? 'followup' : 'new'))
  const newRx = async () => {
    const visit = await todayVisit()
    navigate(`/visits/${visit.id}/prescription`)
  }
  // Repeat ℞ (PM-10): today's prescription opens with that visit's medicines copied in.
  const repeat = async (from: VisitSummary) => {
    const visit = await todayVisit()
    navigate(`/visits/${visit.id}/prescription${visit.id === from.id ? '' : `?repeat=${from.id}`}`)
  }

  const real = (visits.data ?? []).filter((v) => v.status !== 'cancelled')
  const lastVisit = real.find((v) => v.visit_date < todayIso() || v.status === 'seen')
  const nextFollowup = (visits.data ?? [])
    .map((v) => v.prescription?.next_visit_date)
    .filter((d): d is string => !!d && d >= todayIso())
    .sort()[0]
  const counts: Partial<Record<Tab, number>> = {
    visits: real.length,
    reports: reports.data?.length ?? 0,
    prescriptions: (visits.data ?? []).filter((v) => v.prescription).length,
  }
  const tabMeta: Record<Tab, { icon: ComponentType<{ className?: string }>; label: string }> = {
    visits: { icon: CalendarDays, label: t('visits.title') },
    prescriptions: { icon: FileText, label: t('rxtools.prescriptions') },
    vitals: { icon: Activity, label: t('vitals.title') },
    reports: { icon: FlaskConical, label: t('reports.title') },
    ai: { icon: Sparkles, label: t('ai.summaryTitle') },
    billing: { icon: Wallet, label: t('rxtools.billingTab') },
  }

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center gap-3">
        <nav className="flex items-center gap-1.5 text-sm text-slate-500" aria-label="Breadcrumb">
          <Link to="/patients" className="hover:text-brand">
            {t('nav.patients')}
          </Link>
          <ChevronRight className="h-4 w-4" />
          <span className="font-medium text-slate-900">{t('profile.title')}</span>
        </nav>
        <div className="ml-auto flex flex-wrap items-center gap-2">
          <div className="flex items-center gap-1 rounded-xl border border-slate-200 bg-white p-1 shadow-xs">
            <IconAction icon={Pencil} label={t('profile.edit')} onClick={() => setEditing(true)} />
            <IconAction icon={CalendarPlus} label={t('appointments.book')} onClick={() => navigate(`/appointments?book=${id}`)} />
            {role === 'admin' && <IconAction icon={Merge} label={t('rxtools.mergeTitle')} onClick={() => setMerging(true)} danger />}
          </div>
          {role === 'doctor' ? (
            <button onClick={newRx} disabled={startVisit.isPending} className="btn btn-primary">
              <FilePlus2 className="h-4 w-4" /> {openToday ? t('profile.openTodayRx') : t('profile.newRx')}
            </button>
          ) : (
            <button
              onClick={() => startVisit.mutate(visits.data?.length ? 'followup' : 'new')}
              disabled={startVisit.isPending || !!openToday}
              className="btn btn-primary"
            >
              {openToday ? <Check className="h-4 w-4" /> : <ListPlus className="h-4 w-4" />}{' '}
              {openToday ? t('profile.addedToQueue') : t('profile.addToQueue')}
            </button>
          )}
        </div>
      </div>
      {startVisit.isError && <p className="text-sm text-red-600">{startVisit.error.message}</p>}

      <PatientInfoBar patient={p} allergies={profile.data?.allergies ?? []}>
        {openToday && (
          <span className="inline-flex items-center gap-1.5 rounded-full bg-amber-50 px-2.5 py-0.5 text-xs font-medium text-amber-800 ring-1 ring-amber-200">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-amber-500" />
            {t('profile.inQueueToday')}
          </span>
        )}
      </PatientInfoBar>

      {clinicalAccess && (
        <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
          <MiniStat icon={CalendarDays} label={t('profile.totalVisits')} value={visits.data ? real.length : '—'} />
          <MiniStat icon={Clock} label={t('profile.lastVisit')} value={lastVisit ? fmtDate(lastVisit.visit_date) : '—'} />
          <MiniStat icon={BellRing} label={t('profile.nextFollowup')} value={nextFollowup ? fmtDate(nextFollowup) : '—'} tone="violet" />
          <MiniStat
            icon={CalendarCheck}
            label={t('appointments.upcoming')}
            value={upcoming[0] ? `${fmtDate(upcoming[0].date)} · ${upcoming[0].time}` : '—'}
            tone="green"
            onClick={upcoming[0] ? () => navigate(`/appointments?view=day&date=${upcoming[0].date}`) : undefined}
          />
        </div>
      )}

      {clinicalAccess ? (
        <div className="grid gap-5 lg:grid-cols-[300px_1fr]">
          {profile.data ? (
            <MedicalProfilePanel patientId={id} profile={profile.data} sex={p.sex} />
          ) : (
            <aside className="rounded-2xl bg-panel p-4 text-sm text-slate-500">{t('common.loading')}</aside>
          )}
          <section className="min-w-0 rounded-2xl border border-slate-200/80 bg-white shadow-sm">
            <div className="flex gap-1 overflow-x-auto border-b border-slate-100 px-3" role="tablist">
              {tabs.map((k) => {
                const { icon: Icon, label } = tabMeta[k]
                const active = tab === k
                return (
                  <button
                    key={k}
                    role="tab"
                    aria-selected={active}
                    onClick={() => setTab(k)}
                    className={`-mb-px flex shrink-0 items-center gap-2 border-b-2 px-3 py-3 text-sm font-medium whitespace-nowrap ${
                      active ? 'border-brand text-brand' : 'border-transparent text-slate-500 hover:text-slate-800'
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    {label}
                    {counts[k] !== undefined && counts[k]! > 0 && (
                      <span className={`rounded-full px-1.5 text-xs ${active ? 'bg-brand-50 text-brand' : 'bg-slate-100 text-slate-500'}`}>
                        {counts[k]}
                      </span>
                    )}
                  </button>
                )
              })}
            </div>
            <div className="p-5">
              {tab === 'vitals' ? (
                <VitalsTab patientId={id} />
              ) : tab === 'reports' ? (
                <ReportsTab patientId={id} clinical={role === 'doctor'} />
              ) : tab === 'prescriptions' ? (
                <PrescriptionsTab visits={visits.data ?? []} reports={reports.data ?? []} />
              ) : tab === 'billing' ? (
                <BillingTab patientId={id} />
              ) : tab === 'ai' ? (
                <AiSummary patientId={id} available={aiAvailable} />
              ) : visits.data ? (
                <VisitTimeline
                  visits={visits.data}
                  canOpen={role === 'doctor'}
                  onRepeat={role === 'doctor' ? repeat : undefined}
                  reports={reports.data ?? []}
                  showValues={role === 'doctor'}
                />
              ) : (
                <p className="text-slate-500">{t('common.loading')}</p>
              )}
            </div>
          </section>
        </div>
      ) : (
        <>
          <p className="text-sm text-slate-500">{t('profile.noClinicalAccess')}</p>
          {/* Admins collect fees too (§4), so they see the patient's bills. */}
          <Card title={t('rxtools.billingTab')}>
            <BillingTab patientId={id} />
          </Card>
        </>
      )}

      {editing && <PatientFormDialog patient={p} onClose={() => setEditing(false)} />}
      {merging && <MergeDialog patient={p} onClose={() => setMerging(false)} />}
    </div>
  )
}

function IconAction({
  icon: Icon,
  label,
  onClick,
  danger,
}: {
  icon: ComponentType<{ className?: string }>
  label: string
  onClick: () => void
  danger?: boolean
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={label}
      className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm ${
        danger ? 'text-red-600 hover:bg-red-50' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
      }`}
    >
      <Icon className="h-4 w-4" />
      <span className="hidden sm:inline">{label}</span>
    </button>
  )
}

const miniTones = {
  blue: 'bg-brand-50 text-brand',
  violet: 'bg-violet-50 text-violet-600',
  green: 'bg-emerald-50 text-emerald-600',
}

function MiniStat({
  icon: Icon,
  label,
  value,
  tone = 'blue',
  onClick,
}: {
  icon: ComponentType<{ className?: string }>
  label: string
  value: string | number
  tone?: keyof typeof miniTones
  onClick?: () => void
}) {
  const body = (
    <>
      <span className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-lg ${miniTones[tone]}`}>
        <Icon className="h-4 w-4" />
      </span>
      <span className="min-w-0 text-left">
        <span className="block truncate text-xs text-slate-500">{label}</span>
        <span className="block truncate text-sm font-semibold text-slate-900">{value}</span>
      </span>
    </>
  )
  const cls = 'flex items-center gap-3 rounded-xl border border-slate-200/80 bg-white px-3 py-2.5 shadow-xs'
  return onClick ? (
    <button onClick={onClick} className={`${cls} hover:border-brand/40`}>
      {body}
    </button>
  ) : (
    <div className={cls}>{body}</div>
  )
}
