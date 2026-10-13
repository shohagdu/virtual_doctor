import { useMemo, useState, type ComponentType } from 'react'
import { ClipboardList, FilePen, FileUp, History, Images, Lock, Printer, Save, ScrollText, SkipForward, Sparkles } from 'lucide-react'
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
import { useAiAvailable } from '../ai/api'
import { useClinicSettings } from '../settings/api'
import { AiSummary } from '../ai/AiSummary'
import type { Insert } from '../reports/FindingsPanel'
import { usePatientReports } from '../reports/api'
import { PatientReportsDrawer } from '../reports/PatientReportsDrawer'
import { ReportDrawer } from '../reports/ReportDrawer'
import { SafetyPanel } from '../reports/SafetyPanel'
import { useAddFavorite, useFavorites } from '../rxtools/api'
import { DocumentDialog } from '../rxtools/DocumentDialog'
import { DoseCalculator } from '../rxtools/DoseCalculator'
import { clearDraft, useLocalDraft } from '../rxtools/draft'
import { FavoritesMenu } from '../rxtools/FavoritesMenu'
import { TemplatesDrawer } from '../rxtools/TemplatesDrawer'
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
  const [reportsOpen, setReportsOpen] = useState(false)
  const [galleryOpen, setGalleryOpen] = useState(false)
  const patientReports = usePatientReports(String(patient.id))
  const [summaryOpen, setSummaryOpen] = useState(false)
  const [templatesOpen, setTemplatesOpen] = useState(false)
  const [calcOpen, setCalcOpen] = useState(false)
  const [docOpen, setDocOpen] = useState(false)
  const favorites = useFavorites()
  const addFavorite = useAddFavorite()
  const aiAvailable = useAiAvailable()
  const lists = useClinicSettings().data?.rx_lists

  const initial = useMemo(() => {
    const b = bodyOf(rx)
    const medicines = addMedicines(b.medicines, repeatFrom)
    return { ...b, medicines: medicines.length ? medicines : [emptyLine] }
  }, [rx, repeatFrom])
  const [body, setBody] = useState<PrescriptionBody>(initial)
  // Copied-in medicines are unsaved changes until the doctor saves.
  const [dirty, setDirty] = useState(() => addMedicines(rx.medicines, repeatFrom).length > rx.medicines.length)
  const [allergyAck, setAllergyAck] = useState(false)
  const draft = useLocalDraft(visit.id, rx.id, rx.revision, body, dirty)
  const set = (patch: Partial<PrescriptionBody>) => {
    setBody((b) => ({ ...b, ...patch }))
    setDirty(true)
  }

  // AI suggestions and report short forms the doctor ticked (AI spec FR-34); duplicates skipped.
  const insert = (i: Insert) => {
    const merge = (cur: string[], add: string[]) => [...cur, ...add.filter((x) => !cur.includes(x))]
    // Merged against the latest state, so quick successive inserts all land.
    setBody((b) => ({
      ...b,
      diagnosis: merge(b.diagnosis, i.diagnosis),
      investigations: merge(b.investigations, i.investigations),
      advice: merge(b.advice, i.advice),
      next_plan: merge(b.next_plan, i.next_plan),
    }))
    setDirty(true)
  }

  // A template fills empty sections and adds its medicines (roadmap 2.1).
  const applyTemplate = (tpl: PrescriptionBody) => {
    const merge = (cur: string[], add: string[]) => [...cur, ...add.filter((x) => !cur.includes(x))]
    setBody((b) => ({
      ...b,
      complaints: merge(b.complaints, tpl.complaints),
      examination: merge(b.examination, tpl.examination),
      investigations: merge(b.investigations, tpl.investigations),
      diagnosis: merge(b.diagnosis, tpl.diagnosis),
      treatment_plan: merge(b.treatment_plan, tpl.treatment_plan),
      advice: merge(b.advice, tpl.advice),
      next_plan: merge(b.next_plan, tpl.next_plan),
      medicines: addMedicines(b.medicines, tpl.medicines),
    }))
    setDirty(true)
    setAllergyAck(false)
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
    clearDraft(visit.id)
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

  // Each section offers the clinic's own list (Settings → Prescription lists).
  const section = (key: 'complaints' | 'examination' | 'investigations' | 'diagnosis', title: string) => (
    <ListSection title={title} items={body[key]} onChange={(items) => set({ [key]: items })} suggestions={lists?.[key]} />
  )

  return (
    <>
      <div className="space-y-4 print:hidden">
        <div className="flex flex-wrap items-center gap-3">
          <div>
            <h1 className="text-2xl font-bold tracking-tight text-slate-900">{t('rx.title')}</h1>
            <p className="flex items-center gap-2 text-sm text-slate-500">
              {rx.locked ? <Lock className="h-3.5 w-3.5" /> : <FilePen className="h-3.5 w-3.5" />}
              {rx.id === null ? t('rx.new') : rx.locked ? t('rx.lockedNote', { v: rx.version }) : `${t('rx.draft')} v${rx.version}`}
              {dirty && (
                <span className="inline-flex items-center gap-1 rounded-full bg-amber-50 px-2 py-0.5 text-xs font-medium text-amber-700 ring-1 ring-amber-200">
                  <span className="h-1.5 w-1.5 rounded-full bg-amber-500" />
                  {t('rx.unsaved')}
                </span>
              )}
            </p>
          </div>
          <div className="ml-auto flex flex-wrap items-center gap-2">
            {/* Tools */}
            <div className="flex flex-wrap items-center gap-1 rounded-xl border border-slate-200 bg-white p-1 shadow-xs">
              <ToolButton icon={FileUp} label={t('reports.upload')} onClick={() => setReportsOpen(true)} />
              <ToolButton
                icon={Images}
                label={`${t('reports.patientReports')}${patientReports.data?.length ? ` (${patientReports.data.length})` : ''}`}
                onClick={() => setGalleryOpen(true)}
              />
              {aiAvailable && <ToolButton icon={Sparkles} label={t('ai.summariseHistory')} onClick={() => setSummaryOpen(true)} />}
              <ToolButton icon={ClipboardList} label={t('rxtools.templates')} onClick={() => setTemplatesOpen(true)} />
              <ToolButton icon={ScrollText} label={t('rxtools.certificate')} onClick={() => setDocOpen(true)} />
              <ToolButton
                icon={History}
                label={`${t('rx.history')}${previous.length > 0 ? ` (${previous.length})` : ''}`}
                onClick={() => setDrawer(true)}
              />
            </div>
            {/* Actions */}
            <button onClick={() => doSave()} disabled={busy || !dirty || blockedByAllergy} className="btn btn-secondary">
              <Save className="h-4 w-4" /> {t('rx.save')}
            </button>
            <button onClick={doPrint} disabled={busy || blockedByAllergy} className="btn btn-primary">
              <Printer className="h-4 w-4" /> {t('rx.printPdf')}
            </button>
            {isToday && (
              <button
                onClick={doNext}
                disabled={busy || (dirty && blockedByAllergy)}
                className="btn bg-emerald-600 text-white hover:bg-emerald-700"
              >
                {dirty ? t('rx.saveAndNext') : t('rx.nextPatient')} <SkipForward className="h-4 w-4" />
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

        {draft.offer && (
          <div className="flex flex-wrap items-center gap-3 rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900">
            {t('rxtools.draftFound', { time: new Date(draft.offer.at).toLocaleString() })}
            <button
              onClick={() => {
                setBody(draft.offer!.body)
                setDirty(true)
                draft.taken()
              }}
              className="rounded-md bg-amber-600 px-2 py-0.5 text-white"
            >
              {t('rxtools.restore')}
            </button>
            <button onClick={draft.dismiss} className="rounded-md border border-amber-400 px-2 py-0.5">
              {t('rxtools.discard')}
            </button>
          </div>
        )}

        {error && (
          <div className="flex flex-wrap items-center gap-3 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700">
            {error.message}
            {conflict && (
              <button onClick={() => window.location.reload()} className="rounded-md border border-red-400 px-2 py-0.5">
                {t('rx.reload')}
              </button>
            )}
          </div>
        )}

        {matches.length > 0 && (
          <div className="rounded-md border border-red-400 bg-red-50 p-3 text-sm text-red-800">
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
            <div className="relative">
              {section('investigations', t('rx.investigations'))}
              <button
                type="button"
                onClick={() => setReportsOpen(true)}
                className="absolute top-0 right-6 text-sm"
                title={t('reports.upload')}
                aria-label={t('reports.upload')}
              >
                📄
              </button>
            </div>
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
                <span className="ml-auto" />
                <FavoritesMenu onAdd={(line) => copyMedicines([line])} />
                <button
                  type="button"
                  onClick={() => setCalcOpen(true)}
                  className="rounded-md border px-2 py-1 text-xs"
                  title={t('rxtools.doseCalc')}
                >
                  🧮 {t('rxtools.doseCalcShort')}
                </button>
                {lastRx && (
                  <button
                    type="button"
                    onClick={() => copyMedicines(lastRx.prescription?.medicines ?? [])}
                    className="rounded-md border px-2 py-1 text-xs text-brand"
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
                onFavorite={(line) => addFavorite.mutate(line)}
                isFavorite={(line) =>
                  !!favorites.data?.some(
                    (f) =>
                      f.line.brand.toLowerCase() === line.brand.trim().toLowerCase() &&
                      (f.line.strength ?? '') === (line.strength ?? '') &&
                      (f.line.dose_pattern ?? '') === (line.dose_pattern ?? ''),
                  )
                }
              />
            </div>
            <SafetyPanel visitId={visit.id} medicines={body.medicines} />
            <ListSection title={t('rx.advice')} items={body.advice} onChange={(advice) => set({ advice })} suggestions={lists?.advice} />
            <ListSection
              title={t('rx.nextPlan')}
              items={body.next_plan}
              onChange={(next_plan) => set({ next_plan })}
              suggestions={lists?.next_plan}
            />
            <label className="block text-sm font-semibold">
              {t('rx.nextVisit')}
              <input
                type="date"
                min={todayIso()}
                value={body.next_visit_date ?? ''}
                onChange={(e) => set({ next_visit_date: e.target.value || null })}
                className="mt-1 block rounded-md border px-2 py-1 font-normal"
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
      {templatesOpen && (
        <TemplatesDrawer
          current={body}
          onApply={(tpl) => {
            applyTemplate(tpl)
            setTemplatesOpen(false)
          }}
          onClose={() => setTemplatesOpen(false)}
        />
      )}
      {calcOpen && (
        <DoseCalculator
          weight={ctx.vitals?.weight ?? null}
          lines={body.medicines}
          onApply={(i, notes) => set({ medicines: body.medicines.map((m, j) => (j === i ? { ...m, notes } : m)) })}
          onClose={() => setCalcOpen(false)}
        />
      )}
      {docOpen && <DocumentDialog visit={visit} patient={patient} diagnosis={body.diagnosis} onClose={() => setDocOpen(false)} />}
      {galleryOpen && (
        <PatientReportsDrawer patientId={patient.id} visitId={visit.id} showValues onClose={() => setGalleryOpen(false)} />
      )}
      {reportsOpen && (
        <ReportDrawer
          visitId={visit.id}
          rx={{
            complaints: body.complaints,
            examination: body.examination,
            diagnosis: body.diagnosis,
            investigations: body.investigations,
            medicines: body.medicines,
          }}
          aiAvailable={aiAvailable}
          onInsert={insert}
          onClose={() => setReportsOpen(false)}
        />
      )}
      {summaryOpen && (
        <div className="fixed inset-0 z-30 print:hidden" role="dialog" aria-label={t('ai.summaryTitle')}>
          <div className="absolute inset-0 bg-slate-900/40 backdrop-blur-[2px]" onClick={() => setSummaryOpen(false)} />
          <aside className="absolute inset-y-0 right-0 w-full max-w-xl overflow-y-auto bg-white p-4 shadow-xl">
            <div className="mb-3 flex items-center">
              <h2 className="font-semibold">🤖 {t('ai.summaryTitle')}</h2>
              <button onClick={() => setSummaryOpen(false)} className="ml-auto px-2 text-lg" aria-label={t('common.close')}>
                ✕
              </button>
            </div>
            <AiSummary patientId={String(patient.id)} available={aiAvailable} />
          </aside>
        </div>
      )}
      {vitalsOpen && <VitalsDialog visitId={visit.id} patientId={patient.id} onClose={() => setVitalsOpen(false)} />}
    </>
  )
}

/** A compact toolbar button: icon always, label from medium screens up. */
function ToolButton({ icon: Icon, label, onClick }: { icon: ComponentType<{ className?: string }>; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={label}
      aria-label={label}
      className="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-slate-600 hover:bg-slate-100 hover:text-slate-900"
    >
      <Icon className="h-4 w-4" />
      <span className="hidden xl:inline">{label}</span>
    </button>
  )
}
