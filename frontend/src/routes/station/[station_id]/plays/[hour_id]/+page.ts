import { listPlays } from "$lib/api";
import type { PageLoad } from "./$types";

import { error } from "@sveltejs/kit";
import moment from "moment";

export const load: PageLoad = async ({ fetch, params, parent }) => {
  const hourId = params.hour_id;
  if (!hourId.match(/^\d{4}-\d{2}-\d{2}T\d{2}Z$/)) throw error(404);

  const currentPageHour = moment(hourId.replace("Z", ":00:00Z")).startOf("hour");
  if (!currentPageHour.isValid()) throw error(404);

  const { plays, nextToken, invalidate } = await listPlays({
    fetch,
    stationId: params.station_id,
    start: currentPageHour,
    end: moment(currentPageHour).endOf("hour"),
  });

  const maxPageHour = moment().startOf("hour");

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
