import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";
import { getTrack, listTrackPlays } from "$lib/api";

export const load: PageLoad = async ({ fetch, params, parent }) => {
  const { station_id: stationId, track_id: trackId } = params;

  const layoutData = await parent();

  const station = layoutData.stationMap[stationId];
  if (!station) throw error(404);

  const [trackData, playsData] = await Promise.all([
    getTrack({ fetch, stationId, trackId }),
    listTrackPlays({ fetch, stationId, trackId }),
  ]);

  return {
    station,
    trackData,
    playsData,
  };
};
