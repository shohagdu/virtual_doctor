import { differenceInYears, differenceInMonths } from 'date-fns'

/** Age text from DOB, recalculated on every render (PM-04). */
export function ageText(dob: string): string {
  const d = new Date(dob)
  const years = differenceInYears(new Date(), d)
  if (years >= 1) return `${years} Y`
  return `${differenceInMonths(new Date(), d)} M`
}
