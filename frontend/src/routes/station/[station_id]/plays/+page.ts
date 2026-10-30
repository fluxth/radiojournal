import type { PageLoad } from "./$types";
import dayjs from "#lib/dayjs.js";
import { toHourId } from "#lib/helpers.js";
import { redirect } from "@sveltejs/kit";

export const load: PageLoad = async ({ params }) => {
  const now = dayjs();
  redirect(302, `/station/${params.station_id}/plays/${toHourId(now)}`);
};
