import { useTranslation } from 'react-i18next'
import type { Me, Patient, PrescriptionBody, Visit } from '../../api/types'
import { ageText } from '../../utils/age'
import { fmtDate } from '../../utils/date'

const sexLabel = { M: 'Male', F: 'Female', O: 'Other' } as const

function Section({ title, items }: { title: string; items: string[] }) {
  if (items.length === 0) return null
  return (
    <div className="mb-3">
      <div className="font-semibold">{title}</div>
      <ul className="ml-4 list-disc">
        {items.map((i) => (
          <li key={i}>{i}</li>
        ))}
      </ul>
    </div>
  )
}

/** The printed pad (hidden on screen). Browser "Save as PDF" gives the PDF copy. */
export function PrintView({
  me,
  patient,
  visit,
  body,
  version,
}: {
  me: Me | undefined
  patient: Patient
  visit: Visit
  body: PrescriptionBody
  version: number
}) {
  const { t } = useTranslation()
  return (
    <div className="hidden text-[13px] leading-snug text-black print:block">
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

      <div className="flex flex-wrap justify-between gap-x-4 border-b border-black py-1.5">
        <span>
          <b>Name:</b> {[patient.title, patient.name_en].filter(Boolean).join(' ')}
        </span>
        <span>
          <b>Age:</b> {ageText(patient.dob)}
        </span>
        <span>
          <b>Sex:</b> {sexLabel[patient.sex]}
        </span>
        <span>
          <b>ID:</b> {patient.patient_code}
        </span>
        <span>
          <b>Date:</b> {fmtDate(visit.visit_date)}
        </span>
      </div>

      <div className="grid grid-cols-[35%_1fr] gap-4 pt-3">
        <div className="border-r border-black pr-3">
          <Section title={t('rx.complaints')} items={body.complaints} />
          <Section title={t('rx.examination')} items={body.examination} />
          <Section title={t('rx.investigations')} items={body.investigations} />
          <Section title={t('rx.diagnosis')} items={body.diagnosis} />
        </div>
        <div>
          <div className="mb-2 text-2xl font-bold">℞</div>
          <ol className="mb-4 ml-5 list-decimal space-y-2">
            {body.medicines.filter((m) => m.brand.trim()).map((m, i) => (
              <li key={i}>
                <b>{[m.form, m.brand, m.strength].filter(Boolean).join(' ')}</b>
                {m.generic && <span className="text-[11px]"> ({m.generic})</span>}
                <div>{[m.dose_pattern, m.timing, m.duration].filter(Boolean).join(' — ')}</div>
                {m.notes && <div className="text-[12px]">{m.notes}</div>}
              </li>
            ))}
          </ol>
          <Section title={t('rx.advice')} items={body.advice} />
          {body.next_visit_date && (
            <div className="mt-2">
              <b>{t('rx.nextVisit')}:</b> {fmtDate(body.next_visit_date)}
            </div>
          )}
        </div>
      </div>

      <footer className="mt-16 flex items-end justify-between text-[11px]">
        <span>
          Rx v{version} · {patient.patient_code}
        </span>
        <span className="border-t border-black px-8 pt-1 text-[13px]">{t('rx.signature')}</span>
      </footer>
    </div>
  )
}
