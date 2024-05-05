<script lang="ts">
  import type { PageData } from "./$types";
  import { listTrackPlays, type Track } from "$lib/api";
  import dayjs from "$lib/dayjs";
  import { toHourId } from "$lib/helpers";

  import Chart from "chart.js/auto";
  import { onDestroy, onMount } from "svelte";

  export let data: PageData;

  $: track = data.trackData.track;

  $: playsData = data.playsData;
  $: plays = playsData.plays;

  const numberFormat = new Intl.NumberFormat();

  let chartCanvas: HTMLCanvasElement;

  let chart: Chart | null = null;

  $: if (chart) {
    const chartData: number[] = [];
    if (plays.length > 0)
      playsData.plays.reduce((prev, val) => {
        const diff = dayjs(prev.played_at).diff(val.played_at);
        chartData.push(diff / 1000 / 60 / 60);

        return val;
      });
    const chartLabels = chartData.map((_val, idx) => `${-1 * (idx + 1)} play`);

    chart.data.labels = chartLabels;
    chart.data.datasets[0].data = chartData;
    chart.update();
  }

  onMount(() => {
    chart = new Chart(chartCanvas, {
      type: "line",
      data: {
        labels: [],
        datasets: [
          {
            label: "Play gap in hours",
            data: [],
            tension: 0.1,
          },
        ],
      },
      options: {
        scales: {
          y: {
            beginAtZero: true,
            ticks: {
              callback: function (value) {
                if (typeof value === "string") return value;

                if (value >= 24) {
                  const days = Math.floor(value / 24);
                  const hours = value % 24;

                  if (hours === 0) return days + "d";
                  return `${days}d ${hours}h`;
                }

                return value + "h";
              },
            },
          },
        },
      },
    });
  });

  onDestroy(() => {
    if (chart) chart.destroy();
  });

  const refresh = async () => {
    await Promise.allSettled([data.trackData.invalidate(), playsData.invalidate()]);
  };

  const getTrackType = (track: Track): string => {
    return track.is_song ? "Music" : "Other";
  };

  const loadMorePlays = async () => {
    const { plays, ...rest } = await listTrackPlays({
      stationId: data.station.id,
      trackId: track.id,
      nextToken: playsData.nextToken,
    });

    playsData = {
      plays: [...playsData.plays, ...plays],
      ...rest,
    };
  };
</script>

<svelte:head>
  <title>{track.artist} / {track.title} - {data.station.name} - radiojournal</title>
</svelte:head>

<div class="px-2 py-6 flex flex-wrap gap-4">
  <h2 class="font-bold text-2xl truncate">{data.station.name}</h2>
  <button class="btn btn-sm" on:click={refresh}>Refresh</button>
</div>

<div class="text-sm breadcrumbs px-4 bg-base-200 rounded-md">
  <ul>
    <li><a href="/">Stations</a></li>
    <li><a href={`/station/${data.station.id}/plays`}>{data.station.name}</a></li>
    <li><a href={`/station/${data.station.id}/tracks`}>Tracks</a></li>
    <li>{track.artist}</li>
    <li>{track.title}</li>
  </ul>
</div>

<div class="lg:flex gap-4 my-4">
  <div class="flex-1">
    <div>
      <h2 class="font-bold text-2xl truncate mx-2 mb-4">Track Details</h2>

      <div class="stats stats-vertical lg:stats-horizontal shadow-lg mb-4">
        <div class="stat">
          <div class="stat-title">Play Count</div>
          <div class="stat-value">{numberFormat.format(track.play_count)}</div>
        </div>

        <div class="stat">
          <div class="stat-title">First Played</div>
          <div class="stat-value">{dayjs(track.created_at).fromNow()}</div>
          <div class="stat-desc">
            {dayjs(track.created_at).format("ddd MMM DD, YYYY [at] HH:mm")}
          </div>
        </div>

        <div class="stat">
          <div class="stat-title">Last Played</div>
          <div class="stat-value">{dayjs(track.updated_at).fromNow()}</div>
          <div class="stat-desc">
            {dayjs(track.updated_at).format("ddd MMM DD, YYYY [at] HH:mm")}
          </div>
        </div>
      </div>

      <div class="overflow-x-auto mb-4">
        <table class="table table-fixed">
          <tbody>
            <tr>
              <td class="font-bold w-24">Track ID</td>
              <td>
                {track.id}
              </td>
            </tr>
            <tr>
              <td class="font-bold w-24">Artist</td>
              <td>
                {track.artist}
              </td>
            </tr>
            <tr>
              <td class="font-bold w-24">Title</td>
              <td>
                {track.title}
              </td>
            </tr>
            <tr>
              <td class="font-bold w-24">Type</td>
              <td>
                {getTrackType(track)}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <canvas class="mb-4" bind:this={chartCanvas}></canvas>

      <div class="text-right">
        <button class="btn btn-sm" disabled={!playsData.nextToken} on:click={loadMorePlays}>
          Load More
        </button>
      </div>
    </div>
  </div>

  <div class="basis-1/3">
    <ul class="timeline timeline-vertical max-md:-ml-20">
      {#each plays as play, i}
        <li>
          {#if i > 0}
            <hr />
          {/if}
          <div class="timeline-end text-sm italic mx-4 my-2 text-neutral-300 dark:text-neutral-600">
            {#if i > 0}
              {dayjs.duration(dayjs(plays[i - 1].played_at).diff(play.played_at)).humanize()}
              apart
            {/if}
          </div>
          <hr />
        </li>
        <li>
          <hr />
          <div class="timeline-start">{dayjs(play.played_at).fromNow()}</div>
          <div class="timeline-middle">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              class="w-5 h-5"
              ><path
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
                clip-rule="evenodd"
              /></svg
            >
          </div>
          <div class="timeline-end timeline-box">
            <a
              href={`/station/${data.station.id}/plays/${toHourId(dayjs(play.played_at))}`}
              class="link"
            >
              {dayjs(play.played_at).format("ddd MMM DD, YYYY [at] HH:mm")}
            </a>
          </div>
          <hr />
        </li>
      {/each}
    </ul>
  </div>
</div>
