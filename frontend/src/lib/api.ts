import { invalidate } from "$app/navigation";
import type { Dayjs } from "dayjs";

const API_BASE_URL = "https://nna6sr3v62fsk5oauz37tnzuvm0lvtfl.lambda-url.ap-southeast-1.on.aws";

// TODO import from openapi
export type Station = {
  id: string;
  name: string;
  location?: string;
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

export type Track = TrackMinimal & {
  play_count: number;
  created_at: string;
  updated_at: string;
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
  start: Dayjs;
  end: Dayjs;
  nextToken?: string | null;
}): Promise<PlayResponse> => {
  if (!fetch) fetch = window.fetch;

  const params = new URLSearchParams();

  params.append("start", start.toISOString());
  params.append("end", end.toISOString());
  if (nextToken) params.append("next_token", nextToken);

  const url = `${API_BASE_URL}/v1/station/${stationId}/plays?${params.toString()}`;

  const res = await fetch(url);
  const data = await res.json();

  return {
    plays: data.plays,
    nextToken: data.next_token,
    invalidate: async () => await invalidate(url),
  };
};

export type TrackResponse = {
  track: Track;
  invalidate: () => Promise<void>;
};

export const getTrack = async ({
  fetch,
  stationId,
  trackId,
}: {
  fetch?: typeof window.fetch;
  stationId: string;
  trackId: string;
}): Promise<TrackResponse> => {
  if (!fetch) fetch = window.fetch;

  const url = `${API_BASE_URL}/v1/station/${stationId}/track/${trackId}`;
  const res = await fetch(url);

  const track: Track = await res.json();
  return {
    track,
    invalidate: async () => await invalidate(url),
  };
};

export type TrackPlay = {
  played_at: string;
};

export type TrackPlayResponse = {
  plays: TrackPlay[];
  nextToken: string | null;
  invalidate: () => Promise<void>;
};

export const listTrackPlays = async ({
  fetch,
  stationId,
  trackId,
  nextToken,
}: {
  fetch?: typeof window.fetch;
  stationId: string;
  trackId: string;
  nextToken?: string | null;
}): Promise<TrackPlayResponse> => {
  if (!fetch) fetch = window.fetch;

  const params = new URLSearchParams();

  if (nextToken) params.append("next_token", nextToken);

  const paramsEncoded = params.size ? `?${params.toString()}` : "";

  const url = `${API_BASE_URL}/v1/station/${stationId}/track/${trackId}/plays${paramsEncoded}`;
  const res = await fetch(url);

  const { plays, next_token } = await res.json();
  return {
    plays,
    nextToken: next_token,
    invalidate: async () => await invalidate(url),
  };
};
