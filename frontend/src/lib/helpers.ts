import type { Dayjs } from "dayjs";

export const toHourId = (date: Dayjs): string =>
  date.startOf("hour").utc().format("YYYY-MM-DD[T]HH[Z]");
