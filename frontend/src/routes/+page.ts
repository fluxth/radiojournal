import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ parent }) => {
  const layoutData = await parent();

  return {
    stations: Object.values(layoutData.stationMap),
  };
};
