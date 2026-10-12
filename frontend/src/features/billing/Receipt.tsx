import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import { format } from 'date-fns'
import { useMe } from '../../api/me'
import type { Bill, PatientBrief, VisitType } from '../../api/types'
import { bnDigits, taka } from '../../utils/money'
import { fullName } from '../../utils/patient'

/**
 * Money receipt (§5.8), printed in English or Bangla whatever the screen language.
 * Rendered straight into <body> and hidden on screen; while it exists, printing
 * shows only the receipt (see index.css).
 */
export function Receipt({
  bill,
  patient,
  visitDate,
  visitType,
  lang,
}: {
  bill: Bill
  patient: PatientBrief
  visitDate: string
  visitType: VisitType
  lang: 'en' | 'bn'
}) {
  const { i18n } = useTranslation()
  const t = i18n.getFixedT(lang)
  const { data: me } = useMe()
  const n = (s: string) => (lang === 'bn' ? bnDigits(s) : s)
  const money = (x: number) => n(taka(x))
  const name = lang === 'bn' && patient.name_bn ? patient.name_bn : fullName(patient)

  const rows: [string, string][] = [
    [t(`visits.type.${visitType}`), money(bill.fee)],
    ...(bill.discount > 0
      ? [[`${t('billing.discount')}${bill.discount_reason ? ` (${bill.discount_reason})` : ''}`, `− ${money(bill.discount)}`] as [string, string]]
      : []),
  ]

  return createPortal(
    <div className="print-sheet hidden text-[13px] text-black print:block">
      <div className="mx-auto max-w-[120mm] space-y-3">
        <header className="border-b pb-2 text-center">
          <div className="text-lg font-semibold">{me?.clinic.name}</div>
          {me?.clinic.address && <div>{me.clinic.address}</div>}
          {me?.clinic.phone && <div>{n(me.clinic.phone)}</div>}
          <div className="mt-1 font-semibold tracking-wide">{t('billing.receipt')}</div>
        </header>
        <div className="grid grid-cols-2 gap-y-0.5">
          <span>{t('billing.receiptNo')}: {n(bill.receipt_no)}</span>
          <span className="text-right">{t('billing.date')}: {n(format(new Date(visitDate), 'dd/MM/yyyy'))}</span>
          <span>{t('patients.name')}: {name}</span>
          <span className="text-right">{t('patients.id')}: {n(patient.patient_code)}</span>
        </div>
        <table className="w-full">
          <tbody>
            {rows.map(([label, value]) => (
              <tr key={label} className="border-b">
                <td className="py-1">{label}</td>
                <td className="py-1 text-right">{value}</td>
              </tr>
            ))}
            <tr className="font-semibold">
              <td className="py-1">{t('billing.payable')}</td>
              <td className="py-1 text-right">{money(bill.total)}</td>
            </tr>
            <tr>
              <td className="py-1">
                {t('billing.paid')} ({t(`billing.methods.${bill.method}`)})
              </td>
              <td className="py-1 text-right">{money(bill.paid)}</td>
            </tr>
            {bill.due > 0 && (
              <tr className="font-semibold">
                <td className="py-1">{t('billing.due')}</td>
                <td className="py-1 text-right">{money(bill.due)}</td>
              </tr>
            )}
          </tbody>
        </table>
        <footer className="flex justify-between pt-6 text-xs">
          <span>{bill.due > 0 ? t('billing.statusDue') : t('billing.statusPaid')}</span>
          <span className="border-t px-6 pt-1">{t('billing.receivedBy')}</span>
        </footer>
      </div>
    </div>,
    document.body,
  )
}
