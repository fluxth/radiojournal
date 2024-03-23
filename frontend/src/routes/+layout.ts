import type { LayoutLoad } from "./$types";
import { listStations } from "$lib/api";

export const load: LayoutLoad = async ({ fetch }) => {
  const stations = await listStations({ fetch });

  return {
    stations: stations.reduce((map: any, obj) => {
      map[obj.id] = obj;
      return map;
    }, {}),
  };
};
