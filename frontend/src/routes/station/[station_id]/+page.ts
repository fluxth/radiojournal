import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch, params }) => {
  const res = await fetch(
    `https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws/v1/station/${params.station_id}/plays`,
  );

  const { plays, next_token: nextToken } = await res.json();

  return {
    station: {
      id: params.station_id,
    },
    content: {
      plays,
      nextToken,
    },
  };
};
