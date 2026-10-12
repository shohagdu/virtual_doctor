import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { Bill, PatientBrief, PaymentMethod, VisitType } from '../../api/types'
import { fullName } from '../../utils/patient'
import { taka } from '../../utils/money'
import { useSaveBill, useVisitBill } from './api'
import { Receipt } from './Receipt'

const METHODS: PaymentMethod[] = ['cash', 'bkash', 'nagad', 'card']
const round2 = (n: number) => Math.round(n * 100) / 100

interface Props {
  visitId: number
  visitDate: string
  visitType: VisitType
  patient: PatientBrief
  onClose: () => void
}

/** Consultation fee and payment for one visit (§5.8). Saving again updates it. */
export function BillDialog(props: Props) {
  const { data } = useVisitBill(props.visitId)
  if (!data) return null
  return <BillForm {...props} bill={data.bill} suggestedFee={data.suggested_fee} />
}

function BillForm({
  visitId,
  visitDate,
  visitType,
  patient,
  onClose,
  bill,
  suggestedFee,
}: Props & { bill: Bill | null; suggestedFee: number }) {
  const { t } = useTranslation()
  const save = useSaveBill(visitId)
  const [fee, setFee] = useState(String(bill?.fee ?? suggestedFee))
  const [discountMode, setDiscountMode] = useState<'amount' | 'percent'>('amount')
  const [discountText, setDiscountText] = useState(bill?.discount ? String(bill.discount) : '')
  const [reason, setReason] = useState(bill?.discount_reason ?? '')
  // A new bill defaults to paid in full; editing keeps what was recorded.
  const [paidText, setPaidText] = useState(bill ? String(bill.paid) : '')
  const [method, setMethod] = useState<PaymentMethod>(bill?.method ?? 'cash')
  const [saved, setSaved] = useState<Bill | null>(null)
  const [printLang, setPrintLang] = useState<'en' | 'bn'>('en')

  const feeN = Number(fee) || 0
  const discountRaw = Number(discountText) || 0
  const discount = round2(discountMode === 'percent' ? (feeN * discountRaw) / 100 : discountRaw)
  const total = round2(Math.max(feeN - discount, 0))
  const paid = paidText === '' ? total : Number(paidText) || 0
  const due = round2(Math.max(total - paid, 0))

  const submit = async (e: FormEvent, print: boolean) => {
    e.preventDefault()
    const result = await save.mutateAsync({
      fee: feeN,
      discount,
      discount_reason: discount > 0 ? reason : null,
      paid,
      method,
    })
    setSaved(result)
    if (print) setTimeout(() => window.print(), 100)
  }

  const current = saved ?? bill

  return (
    <div className="fixed inset-0 z-30 flex items-start justify-center overflow-y-auto bg-black/40 p-4 print:hidden">
      <form onSubmit={(e) => submit(e, false)} className="w-full max-w-md space-y-4 rounded-lg bg-white p-6 shadow-lg">
        <div>
          <h2 className="text-lg font-semibold">💳 {t('billing.title')}</h2>
          <p className="text-sm text-slate-500">
            {fullName(patient)} · {patient.patient_code} · {t(`visits.type.${visitType}`)}
            {current && <> · {current.receipt_no}</>}
          </p>
        </div>

        <label className="block text-sm">
          {t('billing.fee')} (৳)
          <input
            type="number"
            min={0}
            step="any"
            value={fee}
            onChange={(e) => setFee(e.target.value)}
            className="mt-1 w-full rounded border px-3 py-2"
            required
          />
        </label>

        <div className="text-sm">
          <div className="flex items-center gap-2">
            {t('billing.discount')}
            <span className="ml-auto flex overflow-hidden rounded border text-xs">
              {(['amount', 'percent'] as const).map((m) => (
                <button
                  key={m}
                  type="button"
                  onClick={() => setDiscountMode(m)}
                  aria-pressed={discountMode === m}
                  className={`px-2 py-0.5 ${discountMode === m ? 'bg-brand text-white' : ''}`}
                >
                  {m === 'amount' ? '৳' : '%'}
                </button>
              ))}
            </span>
          </div>
          <input
            type="number"
            min={0}
            step="any"
            value={discountText}
            onChange={(e) => setDiscountText(e.target.value)}
            placeholder="0"
            className="mt-1 w-full rounded border px-3 py-2"
          />
          {discount > 0 && (
            <input
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder={t('billing.discountReason')}
              maxLength={150}
              required
              className="mt-2 w-full rounded border px-3 py-2"
            />
          )}
        </div>

        <label className="block text-sm">
          {t('billing.paid')} (৳)
          <input
            type="number"
            min={0}
            step="any"
            value={paidText}
            onChange={(e) => setPaidText(e.target.value)}
            placeholder={String(total)}
            className="mt-1 w-full rounded border px-3 py-2"
          />
        </label>

        <fieldset className="text-sm">
          <legend>{t('billing.method')}</legend>
          <div className="mt-1 grid grid-cols-4 gap-2">
            {METHODS.map((m) => (
              <label
                key={m}
                className={`cursor-pointer rounded border px-2 py-1.5 text-center ${method === m ? 'border-brand bg-blue-50 text-brand' : ''}`}
              >
                <input
                  type="radio"
                  name="method"
                  value={m}
                  checked={method === m}
                  onChange={() => setMethod(m)}
                  className="sr-only"
                />
                {t(`billing.methods.${m}`)}
              </label>
            ))}
          </div>
        </fieldset>

        <dl className="grid grid-cols-2 gap-y-1 rounded bg-panel p-3 text-sm">
          <dt>{t('billing.payable')}</dt>
          <dd className="text-right font-semibold">{taka(total)}</dd>
          <dt>{t('billing.due')}</dt>
          <dd className={`text-right font-semibold ${due > 0 ? 'text-amber-700' : ''}`}>{taka(due)}</dd>
          <dt>{t('billing.status')}</dt>
          <dd className="text-right">{due > 0 ? t('billing.statusDue') : `✔ ${t('billing.statusPaid')}`}</dd>
        </dl>

        {save.isError && <p className="text-sm text-red-600">{save.error.message}</p>}
        {saved && <p className="text-sm text-green-700">✔ {t('billing.saved', { no: saved.receipt_no })}</p>}

        <div className="flex flex-wrap items-center justify-end gap-2">
          <select
            value={printLang}
            onChange={(e) => setPrintLang(e.target.value as 'en' | 'bn')}
            className="mr-auto rounded border bg-white px-2 py-2 text-sm"
            aria-label={t('billing.receiptLanguage')}
          >
            <option value="en">Receipt: English</option>
            <option value="bn">রসিদ: বাংলা</option>
          </select>
          <button type="button" onClick={onClose} className="rounded border px-4 py-2">
            {t('common.close')}
          </button>
          <button
            type="button"
            onClick={(e) => submit(e, true)}
            disabled={save.isPending}
            className="rounded border border-brand px-4 py-2 text-brand disabled:opacity-60"
          >
            🖨 {t('billing.saveAndPrint')}
          </button>
          <button disabled={save.isPending} className="rounded bg-brand px-4 py-2 text-white disabled:opacity-60">
            {t('patients.save')}
          </button>
        </div>
      </form>
      {current && <Receipt bill={current} patient={patient} visitDate={visitDate} visitType={visitType} lang={printLang} />}
    </div>
  )
}
