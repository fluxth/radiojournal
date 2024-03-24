import type { LayoutLoad } from "./$types";
import { listStations } from "$lib/api";

export const ssr = false;

export const load: LayoutLoad = async ({ fetch }) => {
  const stations = await listStations({ fetch });

  return {
    stationMap: Object.fromEntries(stations.map((station) => [station.id, station])),
  };
};
