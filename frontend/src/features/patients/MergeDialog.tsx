import { useDeferredValue, useState } from 'react'
import { useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { MergeSide, Patient } from '../../api/types'
import { ageText } from '../../utils/age'
import { fmtDate } from '../../utils/date'
import { fullName } from '../../utils/patient'
import { useMergePatients, useMergePreview } from '../rxtools/api'
import { usePatients } from './api'

/**
 * ⇄ Merge duplicate records (PM-19, Admin): pick the duplicate, preview both
 * histories, choose which record to keep, then merge. Everything moves to the kept
 * record; the other is retired, not deleted, and the merge is logged.
 */
export function MergeDialog({ patient, onClose }: { patient: Patient; onClose: () => void }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [q, setQ] = useState(patient.phone)
  const query = useDeferredValue(q.trim())
  const search = usePatients(query, 1, query.length >= 2)
  const [other, setOther] = useState<number | null>(null)
  const [keepCurrent, setKeepCurrent] = useState(true)
  const [sure, setSure] = useState(false)
  const keep = keepCurrent ? patient.id : (other ?? patient.id)
  const drop = keepCurrent ? other : patient.id
  const preview = useMergePreview(keep, other === null ? null : drop)
  const merge = useMergePatients()

  return (
    <div className="fixed inset-0 z-30 flex items-start justify-center overflow-y-auto bg-black/40 p-4">
      <div className="w-full max-w-3xl space-y-4 rounded-lg bg-white p-6 text-sm shadow-lg">
        <h2 className="text-lg font-semibold">⇄ {t('rxtools.mergeTitle')}</h2>
        {other === null ? (
          <>
            <input
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder={`🔍 ${t('patients.search')}`}
              className="w-full rounded border px-3 py-2"
              autoFocus
            />
            <ul className="max-h-72 divide-y overflow-y-auto rounded border">
              {search.data?.items
                .filter((p) => p.id !== patient.id)
                .map((p) => (
                  <li key={p.id}>
                    <button onClick={() => setOther(p.id)} className="w-full px-3 py-2 text-left hover:bg-slate-50">
                      <b>{fullName(p)}</b>{' '}
                      <span className="text-xs text-slate-500">
                        {p.patient_code} · {ageText(p.dob)} / {p.sex} · {p.phone}
                      </span>
                    </button>
                  </li>
                ))}
              {search.data && search.data.items.filter((p) => p.id !== patient.id).length === 0 && (
                <li className="px-3 py-2 text-slate-500">{t('patients.empty')}</li>
              )}
            </ul>
          </>
        ) : !preview.data ? (
          <p className="text-slate-500">{preview.isError ? preview.error.message : t('common.loading')}</p>
        ) : (
          <>
            <div className="grid gap-3 md:grid-cols-2">
              <SideCard side={preview.data.keep} label={`✔ ${t('rxtools.keep')}`} tone="border-green-500" />
              <SideCard side={preview.data.merge} label={`→ ${t('rxtools.mergeAway')}`} tone="border-slate-300" />
            </div>
            <button onClick={() => setKeepCurrent(!keepCurrent)} className="rounded border px-3 py-1">
              ⇄ {t('rxtools.swap')}
            </button>
            <label className="flex items-start gap-2 rounded bg-amber-50 p-3 text-amber-900">
              <input type="checkbox" checked={sure} onChange={(e) => setSure(e.target.checked)} className="mt-1" />
              {t('rxtools.mergeConfirm', { from: preview.data.merge.patient.patient_code, to: preview.data.keep.patient.patient_code })}
            </label>
          </>
        )}
        {merge.isError && <p className="text-red-600">{merge.error.message}</p>}
        <div className="flex justify-end gap-2">
          {other !== null && (
            <button
              onClick={() => {
                setOther(null)
                setSure(false)
              }}
              className="mr-auto rounded border px-4 py-2"
            >
              ← {t('rxtools.pickAnother')}
            </button>
          )}
          <button onClick={onClose} className="rounded border px-4 py-2">
            {t('patients.cancel')}
          </button>
          <button
            disabled={!sure || drop === null || merge.isPending}
            onClick={async () => {
              await merge.mutateAsync({ keep, merge: drop! })
              onClose()
              navigate(`/patients/${keep}`)
            }}
            className="rounded bg-red-600 px-4 py-2 text-white disabled:opacity-50"
          >
            {t('rxtools.merge')}
          </button>
        </div>
      </div>
    </div>
  )
}

function SideCard({ side, label, tone }: { side: MergeSide; label: string; tone: string }) {
  const { t } = useTranslation()
  const p = side.patient
  return (
    <section className={`space-y-1 rounded border-2 p-3 ${tone}`}>
      <div className="text-xs font-semibold">{label}</div>
      <div>
        <b>{fullName(p)}</b> {p.name_bn && <span className="text-slate-500">({p.name_bn})</span>}
      </div>
      <div className="text-xs text-slate-600">
        {p.patient_code} · {ageText(p.dob)} / {p.sex} · {p.phone} · {t('patients.registered')} {fmtDate(p.created_at)}
      </div>
      <div className="text-xs">
        {t('rxtools.counts', side.counts)}
      </div>
      <ul className="max-h-40 overflow-y-auto text-xs text-slate-600">
        {side.visits.map((v) => (
          <li key={v.id}>
            {fmtDate(v.visit_date)} · {t(`visits.type.${v.visit_type}`)} · {t(`visits.status.${v.status}`)}
          </li>
        ))}
      </ul>
    </section>
  )
}
