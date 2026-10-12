import { useMemo, useState } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { ApiError } from '../../api/client'
import { useMe } from '../../api/me'
import type { MedicineLine, Prescription, PrescriptionBody, VisitContext } from '../../api/types'
import { ListSection } from '../../components/ListSection'
import { PatientInfoBar } from '../../components/PatientInfoBar'
import { fmtDate, todayIso } from '../../utils/date'
import { useVisits } from '../patients/api'
import { useCallNext } from '../queue/api'
import { vitalsParts } from '../vitals/format'
import { VitalsDialog } from '../vitals/VitalsDialog'
import { addMedicines, emptyLine } from './medicine'
import { HistoryDrawer } from './HistoryDrawer'
import { MedicineEditor } from './MedicineEditor'
import { PrintView } from './PrintView'
import {
  useAllergyCheck,
  useLockPrescription,
  usePrescription,
  useSavePrescription,
  useVisitContext,
} from './api'

export function PrescriptionPage() {
  const { id = '' } = useParams()
  const { t } = useTranslation()
  const [params] = useSearchParams()
  const ctx = useVisitContext(id)
  const rx = usePrescription(id)
  const history = useVisits(ctx.data ? String(ctx.data.patient.id) : '', !!ctx.data)

  if (ctx.isError || rx.isError) {
    return <p className="text-red-600">{(ctx.error ?? rx.error)?.message}</p>
  }
  // Repeat ℞ from the profile timeline arrives as ?repeat=<visit id> (PM-10).
  const repeatId = Number(params.get('repeat'))
  if (!ctx.data || !rx.data || (repeatId && !history.data)) {
    return <p className="text-slate-500">{t('common.loading')}</p>
  }
  const repeatFrom = history.data?.find((v) => v.id === repeatId)?.prescription?.medicines ?? []

  // Remounts with fresh state whenever a new revision comes back from the server.
  return (
    <PrescriptionEditor
      key={`${rx.data.id}-${rx.data.revision}-${rx.data.locked}`}
      ctx={ctx.data}
      rx={rx.data}
      repeatFrom={repeatFrom}
    />
  )
}

function bodyOf(rx: Prescription): PrescriptionBody {
  const { complaints, examination, investigations, diagnosis, treatment_plan, advice, next_plan, operation_notes, next_visit_date, medicines } = rx
  return { complaints, examination, investigations, diagnosis, treatment_plan, advice, next_plan, operation_notes, next_visit_date, medicines }
}

function PrescriptionEditor({
  ctx,
  rx,
  repeatFrom,
}: {
  ctx: VisitContext
  rx: Prescription
  repeatFrom: MedicineLine[]
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [params, setParams] = useSearchParams()
  const { data: me } = useMe()
  const { visit, patient, profile } = ctx
  const visitId = String(visit.id)
  const save = useSavePrescription(visitId)
  const lock = useLockPrescription(visitId)
  const callNext = useCallNext()
  const history = useVisits(String(patient.id))
  const previous = (history.data ?? []).filter((v) => v.id !== visit.id && v.status !== 'cancelled')
  const lastRx = previous.find((v) => (v.prescription?.medicines.length ?? 0) > 0)
  const [drawer, setDrawer] = useState(false)
  const [vitalsOpen, setVitalsOpen] = useState(false)

  const initial = useMemo(() => {
    const b = bodyOf(rx)
    const medicines = addMedicines(b.medicines, repeatFrom)
    return { ...b, medicines: medicines.length ? medicines : [emptyLine] }
  }, [rx, repeatFrom])
  const [body, setBody] = useState<PrescriptionBody>(initial)
  // Copied-in medicines are unsaved changes until the doctor saves.
  const [dirty, setDirty] = useState(() => addMedicines(rx.medicines, repeatFrom).length > rx.medicines.length)
  const [allergyAck, setAllergyAck] = useState(false)
  const set = (patch: Partial<PrescriptionBody>) => {
    setBody((b) => ({ ...b, ...patch }))
    setDirty(true)
  }

  const copyMedicines = (copied: MedicineLine[]) => {
    set({ medicines: addMedicines(body.medicines, copied) })
    setAllergyAck(false)
  }

  const allergyCheck = useAllergyCheck(visitId, body.medicines, profile.allergies.length > 0)
  const matches = profile.allergies.length > 0 ? (allergyCheck.data ?? []) : []
  const flagged = new Set(matches.map((m) => m.brand))
  // Acknowledge before saving changes; reprinting an unchanged locked version needs no repeat.
  const blockedByAllergy = matches.length > 0 && !allergyAck && (dirty || !rx.locked)
  const base = rx.id === null ? null : { id: rx.id, revision: rx.revision }
  const busy = save.isPending || lock.isPending || callNext.isPending
  const error = save.error ?? lock.error ?? callNext.error
  const conflict = error instanceof ApiError && error.status === 409

  const doSave = async () => {
    const saved = await save.mutateAsync({ base, body, allergy_ack: allergyAck })
    // The repeat is done once saved; don't copy again on reload.
    if (params.has('repeat')) setParams({}, { replace: true })
    return saved
  }

  // Save & Next Patient (§5.5): only for today's queue.
  const isToday = visit.visit_date === todayIso()
  const doNext = async () => {
    if (dirty) await doSave()
    const { next } = await callNext.mutateAsync(visit.id)
    navigate(next ? `/visits/${next.id}/prescription` : '/dashboard')
  }

  const doPrint = async () => {
    let current = rx
    if (dirty || rx.id === null) current = await doSave()
    if (!current.locked && current.id !== null) {
      await lock.mutateAsync({ id: current.id, revision: current.revision })
    }
    // Let React render the locked version before the print dialog opens.
    setTimeout(() => window.print(), 100)
  }

  const section = (key: 'complaints' | 'examination' | 'investigations' | 'diagnosis', title: string) => (
    <ListSection title={title} items={body[key]} onChange={(items) => set({ [key]: items })} />
  )

  return (
    <>
      <div className="space-y-4 print:hidden">
        <div className="flex flex-wrap items-center gap-2">
          <h1 className="text-lg font-semibold">{t('rx.title')}</h1>
          <span className="text-sm text-slate-500">
            {rx.id === null
              ? t('rx.new')
              : rx.locked
                ? t('rx.lockedNote', { v: rx.version })
                : `${t('rx.draft')} v${rx.version}`}
            {dirty && ` · ${t('rx.unsaved')}`}
          </span>
          <div className="ml-auto flex flex-wrap gap-2">
            <button onClick={() => setDrawer(true)} className="rounded bg-slate-500 px-3 py-1.5 text-sm text-white">
              🕘 {t('rx.history')}
              {previous.length > 0 && ` (${previous.length})`}
            </button>
            <button
              onClick={() => doSave()}
              disabled={busy || !dirty || blockedByAllergy}
              className="rounded bg-brand px-3 py-1.5 text-sm text-white disabled:opacity-50"
            >
              💾 {t('rx.save')}
            </button>
            <button
              onClick={doPrint}
              disabled={busy || blockedByAllergy}
              className="rounded bg-green-600 px-3 py-1.5 text-sm text-white disabled:opacity-50"
            >
              🖨 {t('rx.printPdf')}
            </button>
            {isToday && (
              <button
                onClick={doNext}
                disabled={busy || (dirty && blockedByAllergy)}
                className="rounded bg-indigo-600 px-3 py-1.5 text-sm text-white disabled:opacity-50"
              >
                {dirty ? t('rx.saveAndNext') : t('rx.nextPatient')} ⏭
              </button>
            )}
          </div>
        </div>

        <PatientInfoBar patient={patient} allergies={profile.allergies} linkToProfile>
          <span>
            <b>{t('visits.visitNo', { n: visit.visit_no })}</b>
          </span>
          {ctx.previous_visit_date && (
            <span>
              <b>{t('rx.lastVisit')}:</b> {fmtDate(ctx.previous_visit_date)}
            </span>
          )}
        </PatientInfoBar>

        {error && (
          <div className="flex flex-wrap items-center gap-3 rounded border border-red-300 bg-red-50 p-3 text-sm text-red-700">
            {error.message}
            {conflict && (
              <button onClick={() => window.location.reload()} className="rounded border border-red-400 px-2 py-0.5">
                {t('rx.reload')}
              </button>
            )}
          </div>
        )}

        {matches.length > 0 && (
          <div className="rounded border border-red-400 bg-red-50 p-3 text-sm text-red-800">
            <p className="font-semibold">⚠ {t('rx.allergyWarning')}</p>
            <ul className="ml-5 list-disc">
              {matches.map((m) => (
                <li key={`${m.brand}-${m.allergy}`}>
                  {m.brand} — {t('profile.allergies')}: {m.allergy}
                  {m.class && ` (${t('rx.sameClass', { cls: m.class })})`}
                </li>
              ))}
            </ul>
            <label className="mt-2 flex items-center gap-2">
              <input type="checkbox" checked={allergyAck} onChange={(e) => setAllergyAck(e.target.checked)} />
              {t('rx.allergyConfirm')}
            </label>
          </div>
        )}

        <div className="grid gap-4 md:grid-cols-[300px_1fr]">
          <aside className="space-y-4 rounded-lg bg-panel p-4">
            <div className="space-y-1">
              <div className="flex items-center justify-between text-sm font-semibold">
                <span>{t('vitals.title')}</span>
                <button type="button" onClick={() => setVitalsOpen(true)} className="text-brand" aria-label={t('vitals.record')}>
                  {ctx.vitals ? '✏' : '➕'}
                </button>
              </div>
              {ctx.vitals ? (
                <ul className="text-sm">
                  {vitalsParts(ctx.vitals).map((p) => (
                    <li key={p}>• {p}</li>
                  ))}
                </ul>
              ) : (
                <p className="text-sm text-slate-400">—</p>
              )}
            </div>
            {section('complaints', t('rx.complaints'))}
            {section('examination', t('rx.examination'))}
            {section('investigations', t('rx.investigations'))}
            {section('diagnosis', t('rx.diagnosis'))}
            <ListSection title={t('profile.chronic')} items={profile.chronic_conditions} readOnly />
            <ListSection title={t('rx.drugHistory')} items={profile.long_term_medicines} readOnly />
            {profile.past_history && (
              <div className="text-sm">
                <div className="font-semibold">{t('profile.pastHistory')}</div>
                <p className="whitespace-pre-line">{profile.past_history}</p>
              </div>
            )}
          </aside>

          <section className="space-y-5 rounded-lg bg-white p-4 shadow-sm">
            <div>
              <div className="mb-2 flex flex-wrap items-center gap-2">
                <span className="text-2xl font-bold text-brand">℞</span>
                {lastRx && (
                  <button
                    type="button"
                    onClick={() => copyMedicines(lastRx.prescription?.medicines ?? [])}
                    className="ml-auto rounded border px-2 py-1 text-xs text-brand"
                  >
                    ↻ {t('rx.repeatLast', { date: fmtDate(lastRx.visit_date) })}
                  </button>
                )}
              </div>
              <MedicineEditor
                lines={body.medicines}
                onChange={(medicines) => set({ medicines })}
                flagged={flagged}
                readOnly={false}
              />
            </div>
            <ListSection title={t('rx.advice')} items={body.advice} onChange={(advice) => set({ advice })} />
            <label className="block text-sm font-semibold">
              {t('rx.nextVisit')}
              <input
                type="date"
                min={todayIso()}
                value={body.next_visit_date ?? ''}
                onChange={(e) => set({ next_visit_date: e.target.value || null })}
                className="mt-1 block rounded border px-2 py-1 font-normal"
              />
            </label>
          </section>
        </div>
      </div>

      <PrintView
        me={me}
        patient={patient}
        visit={visit}
        vitals={ctx.vitals}
        body={body}
        version={Math.max(rx.version, 1)}
      />

      {drawer && (
        <HistoryDrawer
          visits={previous}
          onCopy={(m) => {
            copyMedicines(m)
            setDrawer(false)
          }}
          onClose={() => setDrawer(false)}
        />
      )}
      {vitalsOpen && <VitalsDialog visitId={visit.id} patientId={patient.id} onClose={() => setVitalsOpen(false)} />}
    </>
  )
}
