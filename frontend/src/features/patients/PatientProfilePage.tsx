import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { Card } from '../../components/Card'
import { PatientInfoBar } from '../../components/PatientInfoBar'
import { todayIso } from '../../utils/date'
import { useAuth } from '../auth/authStore'
import { useMedicalProfile, usePatient, useStartVisit, useVisits } from './api'
import { MedicalProfilePanel } from './MedicalProfilePanel'
import { PatientFormDialog } from './PatientFormDialog'
import { VisitTimeline } from './VisitTimeline'

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
  const [editing, setEditing] = useState(false)

  if (isLoading || !p) return <p className="text-slate-500">{t('common.loading')}</p>

  // Reuse today's open visit (e.g. one the assistant queued) instead of starting another.
  const openToday = visits.data?.find(
    (v) => v.visit_date === todayIso() && (v.status === 'waiting' || v.status === 'in_consultation'),
  )
  const newRx = async () => {
    const visit = openToday ?? (await startVisit.mutateAsync(visits.data?.length ? 'followup' : 'new'))
    navigate(`/visits/${visit.id}/prescription`)
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold">{t('profile.title')}</h1>
        <div className="ml-auto flex flex-wrap gap-2">
          <button onClick={() => setEditing(true)} className="rounded bg-slate-500 px-3 py-1.5 text-sm text-white">
            ✏ {t('profile.edit')}
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

      {clinicalAccess ? (
        <div className="grid gap-4 md:grid-cols-[280px_1fr]">
          {profile.data ? (
            <MedicalProfilePanel patientId={id} profile={profile.data} sex={p.sex} />
          ) : (
            <aside className="rounded-lg bg-panel p-4 text-sm text-slate-500">{t('common.loading')}</aside>
          )}
          <Card title={t('visits.title')}>
            {visits.data ? (
              <VisitTimeline visits={visits.data} canOpen={role === 'doctor'} />
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
