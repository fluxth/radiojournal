import { listTracks } from "$lib/api";
import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";

export const load: PageLoad = async ({ fetch, params, parent }) => {
  const { station_id: stationId } = params;

  const layoutData = await parent();

  const station = layoutData.stationMap[stationId];
  if (!station) throw error(404);

  const tracksData = await listTracks({ fetch, stationId });

  return {
    station,
    tracksData,
  };
};
