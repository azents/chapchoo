/** A local Intl limitation must not change the stored zone or invent a date. */
export function formatDisplayDate(
  instant: number,
  region: string,
  timeZone: string,
  dateStyle: "long" | "medium",
): string | null {
  try {
    return new Intl.DateTimeFormat(region, {
      dateStyle,
      timeStyle: "short",
      timeZone,
    }).format(instant);
  } catch (error) {
    if (!(error instanceof RangeError)) {
      throw error;
    }
    return null;
  }
}
