import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch, params }) => {
  const res = await fetch(
    "https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws/v1/stations",
  );

  const stations = await res.json();

  return {
    stations,
  };
};
