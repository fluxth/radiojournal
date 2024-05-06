import { listTracks } from "$lib/api";
import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";

export const load: PageLoad = async ({ fetch, params, parent }) => {
  const { station_id: stationId, artist_name: artistName } = params;

  const layoutData = await parent();

  const station = layoutData.stationMap[stationId];
  if (!station) throw error(404);

  const tracksData = await listTracks({ fetch, stationId, artist: artistName });

  return {
    station,
    tracksData,
    artist: { name: artistName },
  };
};
