/** Taka amount as "৳1,250" (paisa shown only when present). */
export function taka(n: number) {
  return `৳${n.toLocaleString('en-IN', { maximumFractionDigits: 2 })}`
}

/** Western digits to Bangla digits (০–৯), for Bangla print-outs. */
export function bnDigits(s: string) {
  return s.replace(/\d/g, (d) => '০১২৩৪৫৬৭৮৯'[Number(d)])
}
