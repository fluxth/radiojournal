import type { Moment } from "moment";

export const toHourId = (date: Moment): string =>
  date.startOf("hour").toISOString().replace(":00:00.000Z", "Z");
