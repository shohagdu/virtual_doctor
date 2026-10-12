import { format } from 'date-fns'

export function fmtDate(d: string) {
  return format(new Date(d), 'dd MMM yyyy')
}

/** Today's date as YYYY-MM-DD in the browser's local time zone. */
export function todayIso() {
  return format(new Date(), 'yyyy-MM-dd')
}
