import { invalidate } from "$app/navigation";
import type { Moment } from "moment";

const API_BASE_URL = "https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws";

// TODO import from openapi
export type Station = {
  id: string;
  name: string;
  play_count: number;
  track_count: number;
};

export const listStations = async ({
  fetch,
}: {
  fetch?: typeof window.fetch;
}): Promise<Station[]> => {
  if (!fetch) fetch = window.fetch;
  const res = await fetch(`${API_BASE_URL}/v1/stations`);

  const stations: Station[] = await res.json();
  return stations;
};

export type PlayResponse = {
  plays: Play[];
  nextToken: string | null;
  invalidate: () => Promise<void>;
};

export type Play = {
  id: string;
  played_at: string;
  track: TrackMinimal;
};

export type TrackMinimal = {
  id: string;
  artist: string;
  title: string;
  is_song: string;
};

export const listPlays = async ({
  fetch,
  stationId,
  start,
  end,
  nextToken,
}: {
  fetch?: typeof window.fetch;
  stationId: string;
  start?: Moment;
  end?: Moment;
  nextToken?: string;
}): Promise<PlayResponse> => {
  if (!fetch) fetch = window.fetch;

  const params = new URLSearchParams();

  if (start) params.append("start", start.toISOString());
  if (end) params.append("end", end.toISOString());
  if (nextToken) params.append("next_token", nextToken);

  const paramsEncoded = params.size ? `?${params.toString()}` : "";
  const url = `${API_BASE_URL}/v1/station/${stationId}/plays${paramsEncoded}`;

  const res = await fetch(url);
  const data = await res.json();

  return {
    plays: data.plays,
    nextToken: data.next_token,
    invalidate: async () => await invalidate(url),
  };
};
