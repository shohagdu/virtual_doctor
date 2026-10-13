import { useState } from 'react'
import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import { useMe } from '../../api/me'
import type { ClinicDocument, DocumentBody, DocumentKind, Patient, Visit } from '../../api/types'
import { ageText } from '../../utils/age'
import { fmtDate, todayIso } from '../../utils/date'
import { fullName } from '../../utils/patient'
import { useCreateDocument, useDocuments } from './api'

const KINDS: DocumentKind[] = ['sick_leave', 'fitness', 'referral']
const empty: DocumentBody = { diagnosis: null, from: null, to: null, referred_to: null, reason: null, summary: null, remarks: null }

/** 📝 Medical certificate, fitness certificate or referral letter for the visit (roadmap 2.9). */
export function DocumentDialog({
  visit,
  patient,
  diagnosis,
  onClose,
}: {
  visit: Visit
  patient: Patient
  diagnosis: string[]
  onClose: () => void
}) {
  const { t } = useTranslation()
  const docs = useDocuments(visit.id)
  const create = useCreateDocument(visit.id)
  const [kind, setKind] = useState<DocumentKind>('sick_leave')
  const [body, setBody] = useState<DocumentBody>({ ...empty, diagnosis: diagnosis.join(', ') || null, from: todayIso() })
  const [printing, setPrinting] = useState<ClinicDocument | null>(null)
  const set = (patch: Partial<DocumentBody>) => setBody({ ...body, ...patch })
  const field = (key: keyof DocumentBody, label: string, type: 'text' | 'date' | 'area' = 'text') => (
    <label className="block">
      {label}
      {type === 'area' ? (
        <textarea value={body[key] ?? ''} onChange={(e) => set({ [key]: e.target.value || null })} rows={3} className="mt-1 w-full rounded-md border px-2 py-1" />
      ) : (
        <input type={type} value={body[key] ?? ''} onChange={(e) => set({ [key]: e.target.value || null })} className="mt-1 w-full rounded-md border px-2 py-1" />
      )}
    </label>
  )

  const print = (d: ClinicDocument) => {
    setPrinting(d)
    setTimeout(() => window.print(), 100)
  }

  return (
    <div className="overlay print:hidden">
      <div className="w-full max-w-lg space-y-3 rounded-lg bg-white p-6 text-sm shadow-lg">
        <h2 className="text-lg font-semibold">📝 {t('rxtools.documents')}</h2>
        <div className="flex gap-1" role="tablist">
          {KINDS.map((k) => (
            <button
              key={k}
              role="tab"
              aria-selected={kind === k}
              onClick={() => setKind(k)}
              className={`rounded-md px-3 py-1 ${kind === k ? 'bg-brand text-white' : 'border'}`}
            >
              {t(`rxtools.kind.${k}`)}
            </button>
          ))}
        </div>
        {kind === 'sick_leave' && (
          <>
            {field('diagnosis', t('rx.diagnosis'))}
            <div className="grid grid-cols-2 gap-2">
              {field('from', t('rxtools.restFrom'), 'date')}
              {field('to', t('rxtools.restTo'), 'date')}
            </div>
          </>
        )}
        {kind === 'fitness' && (
          <>
            {field('diagnosis', t('rxtools.treatedFor'))}
            {field('from', t('rxtools.fitFrom'), 'date')}
          </>
        )}
        {kind === 'referral' && (
          <>
            {field('referred_to', t('rxtools.referredTo'))}
            {field('reason', t('rxtools.reason'))}
            {field('summary', t('rxtools.summary'), 'area')}
          </>
        )}
        {field('remarks', t('rxtools.remarks'))}
        {create.isError && <p className="text-red-600">{create.error.message}</p>}
        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="btn btn-secondary">
            {t('common.close')}
          </button>
          <button
            onClick={() => create.mutate({ kind, body }, { onSuccess: print })}
            disabled={create.isPending}
            className="btn btn-primary"
          >
            🖨 {t('rxtools.saveAndPrint')}
          </button>
        </div>
        {docs.data && docs.data.length > 0 && (
          <div className="border-t pt-2">
            <div className="font-semibold">{t('rxtools.written')}</div>
            <ul>
              {docs.data.map((d) => (
                <li key={d.id} className="flex items-center gap-2 py-0.5">
                  {t(`rxtools.kind.${d.kind}`)} · {fmtDate(d.created_at)}
                  <button onClick={() => print(d)} className="ml-auto text-brand">
                    🖨 {t('billing.print')}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
      {printing && <DocumentPrint doc={printing} patient={patient} visit={visit} />}
    </div>
  )
}

/** The printed document on the doctor's pad; replaces the app on paper while shown. */
function DocumentPrint({ doc, patient, visit }: { doc: ClinicDocument; patient: Patient; visit: Visit }) {
  const { data: me } = useMe()
  const b = doc.body
  const he = patient.sex === 'F' ? 'She' : patient.sex === 'M' ? 'He' : 'The patient'
  const name = fullName(patient)
  const days = b.from && b.to ? Math.round((Date.parse(b.to) - Date.parse(b.from)) / 86400000) + 1 : null
  const title = { sick_leave: 'MEDICAL CERTIFICATE', fitness: 'FITNESS CERTIFICATE', referral: 'REFERRAL LETTER' }[doc.kind]

  return createPortal(
    <div className="print-sheet hidden text-[14px] leading-relaxed text-black print:block">
      <header className="flex justify-between border-b-2 border-black pb-2">
        <div>
          <div className="text-xl font-bold">{me?.user.name}</div>
          {me?.user.degrees && <div>{me.user.degrees}</div>}
          {me?.user.bmdc_reg_no && <div>BMDC Reg. No: {me.user.bmdc_reg_no}</div>}
        </div>
        <div className="text-right">
          <div className="font-semibold">{me?.clinic.name}</div>
          {me?.clinic.address && <div>{me.clinic.address}</div>}
          {me?.clinic.phone && <div>{me.clinic.phone}</div>}
        </div>
      </header>
      <h1 className="my-6 text-center text-lg font-bold tracking-widest underline">{title}</h1>
      <p className="mb-4 text-right">Date: {fmtDate(doc.created_at)}</p>
      {doc.kind === 'referral' ? (
        <div className="space-y-3">
          <p>To: {b.referred_to}</p>
          <p>
            Dear Colleague, I am referring {name} ({ageText(patient.dob)}, {patient.sex}; ID {patient.patient_code}), seen on{' '}
            {fmtDate(visit.visit_date)}, for: <b>{b.reason}</b>.
          </p>
          {b.summary && <p className="whitespace-pre-line">{b.summary}</p>}
          <p>Thank you for seeing {patient.sex === 'F' ? 'her' : patient.sex === 'M' ? 'him' : 'the patient'}.</p>
        </div>
      ) : doc.kind === 'sick_leave' ? (
        <p>
          This is to certify that {name}, aged {ageText(patient.dob)}, was examined by me on {fmtDate(visit.visit_date)} and is
          suffering from <b>{b.diagnosis}</b>. {he} is advised rest for {days} day{days === 1 ? '' : 's'}, from {fmtDate(b.from!)} to{' '}
          {fmtDate(b.to!)}.
        </p>
      ) : (
        <p>
          This is to certify that {name}, aged {ageText(patient.dob)}
          {b.diagnosis ? (
            <>
              , who was under my treatment for <b>{b.diagnosis}</b>,
            </>
          ) : null}{' '}
          is fit to resume normal duties from {fmtDate(b.from!)}.
        </p>
      )}
      {b.remarks && <p className="mt-3">Remarks: {b.remarks}</p>}
      <footer className="mt-24 flex justify-end">
        <div className="border-t border-black px-10 pt-1 text-center">
          {me?.user.name}
          <div className="text-[12px]">Signature &amp; seal</div>
        </div>
      </footer>
    </div>,
    document.body,
  )
}
