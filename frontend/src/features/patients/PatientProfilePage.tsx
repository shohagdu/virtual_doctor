import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { VisitSummary } from '../../api/types'
import { Card } from '../../components/Card'
import { PatientInfoBar } from '../../components/PatientInfoBar'
import { todayIso } from '../../utils/date'
import { useAuth } from '../auth/authStore'
import { VitalsTab } from '../vitals/VitalsTab'
import { useAiAvailable } from '../ai/api'
import { AiSummary } from '../ai/AiSummary'
import { ReportsTab } from '../reports/ReportsTab'
import { usePatientAppointments } from '../appointments/api'
import { fmtDate } from '../../utils/date'
import { useMedicalProfile, usePatient, useStartVisit, useVisits } from './api'
import { MedicalProfilePanel } from './MedicalProfilePanel'
import { PatientFormDialog } from './PatientFormDialog'
import { VisitTimeline } from './VisitTimeline'

type Tab = 'visits' | 'vitals' | 'reports' | 'ai'

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
  const appointments = usePatientAppointments(id)
  const upcoming = appointments.data?.filter((a) => a.status === 'booked') ?? []
  const [editing, setEditing] = useState(false)
  const [tab, setTab] = useState<Tab>('visits')
  const aiAvailable = useAiAvailable()
  const tabs: Tab[] = role === 'doctor' ? ['visits', 'vitals', 'reports', 'ai'] : ['visits', 'vitals', 'reports']

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

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold">{t('profile.title')}</h1>
        <div className="ml-auto flex flex-wrap gap-2">
          <button onClick={() => setEditing(true)} className="rounded bg-slate-500 px-3 py-1.5 text-sm text-white">
            ✏ {t('profile.edit')}
          </button>
          <button
            onClick={() => navigate(`/appointments?book=${id}`)}
            className="rounded border border-brand bg-white px-3 py-1.5 text-sm text-brand"
          >
            📅 {t('appointments.book')}
          </button>
          {role === 'doctor' ? (
            <button
              onClick={newRx}
              disabled={startVisit.isPending}
              className="rounded bg-brand px-3 py-1.5 text-sm text-white disabled:opacity-60"
            >
              ➕ {openToday ? t('profile.openTodayRx') : t('profile.newRx')}
            </button>
          ) : (
            <button
              onClick={() => startVisit.mutate(visits.data?.length ? 'followup' : 'new')}
              disabled={startVisit.isPending || !!openToday}
              className="rounded bg-brand px-3 py-1.5 text-sm text-white disabled:opacity-60"
            >
              {openToday ? `✔ ${t('profile.addedToQueue')}` : `➕ ${t('profile.addToQueue')}`}
            </button>
          )}
        </div>
      </div>
      {startVisit.isError && <p className="text-sm text-red-600">{startVisit.error.message}</p>}

      <PatientInfoBar patient={p} allergies={profile.data?.allergies ?? []} />

      {upcoming.length > 0 && (
        <p className="rounded bg-blue-50 px-3 py-2 text-sm text-blue-900">
          📅 {t('appointments.upcoming')}:{' '}
          {upcoming.map((a, i) => (
            <span key={a.id}>
              {i > 0 && ', '}
              <button
                onClick={() => navigate(`/appointments?view=day&date=${a.date}`)}
                className="font-medium underline"
              >
                {fmtDate(a.date)} {a.time}
              </button>
            </span>
          ))}
        </p>
      )}

      {clinicalAccess ? (
        <div className="grid gap-4 md:grid-cols-[280px_1fr]">
          {profile.data ? (
            <MedicalProfilePanel patientId={id} profile={profile.data} sex={p.sex} />
          ) : (
            <aside className="rounded-lg bg-panel p-4 text-sm text-slate-500">{t('common.loading')}</aside>
          )}
          <Card
            title={
              <span className="flex flex-wrap gap-1" role="tablist">
                {tabs.map((k) => (
                  <button
                    key={k}
                    role="tab"
                    aria-selected={tab === k}
                    onClick={() => setTab(k)}
                    className={`rounded px-3 py-1 text-sm ${tab === k ? 'bg-brand text-white' : 'text-slate-600 hover:bg-slate-100'}`}
                  >
                    {k === 'ai' ? `🤖 ${t('ai.summaryTitle')}` : t(`${k}.title`)}
                  </button>
                ))}
              </span>
            }
          >
            {tab === 'vitals' ? (
              <VitalsTab patientId={id} />
            ) : tab === 'reports' ? (
              <ReportsTab patientId={id} clinical={role === 'doctor'} />
            ) : tab === 'ai' ? (
              <AiSummary patientId={id} available={aiAvailable} />
            ) : visits.data ? (
              <VisitTimeline
                visits={visits.data}
                canOpen={role === 'doctor'}
                onRepeat={role === 'doctor' ? repeat : undefined}
              />
            ) : (
              <p className="text-slate-500">{t('common.loading')}</p>
            )}
          </Card>
        </div>
      ) : (
        <p className="text-sm text-slate-500">{t('profile.noClinicalAccess')}</p>
      )}

      {editing && <PatientFormDialog patient={p} onClose={() => setEditing(false)} />}
    </div>
  )
}
