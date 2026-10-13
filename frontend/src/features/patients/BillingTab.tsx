import { useTranslation } from 'react-i18next'
import { fmtDate } from '../../utils/date'
import { taka } from '../../utils/money'
import { usePatientBills } from '../rxtools/api'

/** Profile → Billing (§5.4): consultation fees, payments and dues. */
export function BillingTab({ patientId }: { patientId: string }) {
  const { t } = useTranslation()
  const { data } = usePatientBills(patientId)
  if (!data) return <p className="text-slate-500">{t('common.loading')}</p>
  if (data.length === 0) return <p className="text-slate-500">{t('billing.none')}</p>
  const due = data.reduce((s, r) => s + r.due, 0)
  return (
    <div className="space-y-2 text-sm">
      {due > 0 && <p className="rounded-md bg-amber-50 px-3 py-2 font-medium text-amber-800">{t('rxtools.totalDue', { amount: taka(due) })}</p>}
      <div className="overflow-x-auto">
        <table className="w-full text-left">
          <thead className="border-b text-slate-500">
            <tr>
              <th className="py-2 pr-3">{t('billing.date')}</th>
              <th className="py-2 pr-3">{t('billing.receiptNo')}</th>
              <th className="py-2 pr-3">{t('queue.type')}</th>
              <th className="py-2 pr-3 text-right">{t('billing.payable')}</th>
              <th className="py-2 pr-3 text-right">{t('billing.paid')}</th>
              <th className="py-2 pr-3 text-right">{t('billing.due')}</th>
              <th className="py-2 pr-3">{t('billing.method')}</th>
            </tr>
          </thead>
          <tbody>
            {data.map((r) => (
              <tr key={r.bill_id} className="border-b last:border-0">
                <td className="py-2 pr-3 whitespace-nowrap">{fmtDate(r.visit_date)}</td>
                <td className="py-2 pr-3">{r.receipt_no}</td>
                <td className="py-2 pr-3">{t(`visits.type.${r.visit_type}`)}</td>
                <td className="py-2 pr-3 text-right">{taka(r.total)}</td>
                <td className="py-2 pr-3 text-right">{taka(r.paid)}</td>
                <td className={`py-2 pr-3 text-right ${r.due > 0 ? 'font-medium text-amber-700' : ''}`}>{r.due > 0 ? taka(r.due) : '—'}</td>
                <td className="py-2 pr-3">{t(`billing.methods.${r.method}`)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
