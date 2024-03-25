import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";
import { listPlays } from "$lib/api";
import dayjs from "$lib/dayjs";

export const load: PageLoad = async ({ fetch, params, parent }) => {
  const hourId = params.hour_id;
  if (!hourId.match(/^\d{4}-\d{2}-\d{2}T\d{2}Z$/)) throw error(404);

  const currentPageHour = dayjs(hourId.replace("Z", ":00:00Z")).startOf("hour");
  if (!currentPageHour.isValid()) throw error(404);

  const { plays, nextToken, invalidate } = await listPlays({
    fetch,
    stationId: params.station_id,
    start: currentPageHour,
    end: dayjs(currentPageHour).endOf("hour"),
  });

  const maxPageHour = dayjs().startOf("hour");

  const layoutData = await parent();

  const station = layoutData.stationMap[params.station_id];
  if (!station) throw error(404);

  return {
    station,
    pageHour: {
      current: currentPageHour,
      max: maxPageHour,
    },
    content: {
      plays,
      nextToken,
    },
    invalidate,
  };
};
