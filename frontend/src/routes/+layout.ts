import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ fetch }) => {
  const res = await fetch(
    "https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws/v1/stations",
  );

  const stations: any[] = await res.json();

  return {
    stations: stations.reduce((map, obj) => {
      map[obj.id] = obj;
      return map;
    }, {}),
  };
};
