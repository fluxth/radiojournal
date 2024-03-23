<script lang="ts">
  import { goto, invalidate } from "$app/navigation";
  import { toHourId } from "$lib/helpers";
  import moment from "moment";

  export let data: any;

  const refresh = async () => {
    await data.invalidate();
  };
</script>

<svelte:head>
  <title>{data.station.name} - radiojournal</title>
</svelte:head>

<div class="px-2 py-6 flex flex-wrap gap-4">
  <h2 class="font-bold text-2xl truncate">{data.station.name}</h2>
  <button class="btn btn-sm" on:click={refresh}>Refresh</button>
</div>

<div class="text-sm breadcrumbs px-4 bg-base-200 rounded-md">
  <ul>
    <li><a href="/">Stations</a></li>
    <li>{data.station.name}</li>
  </ul>
</div>

<div class="my-4 flex flex-col items-center gap-1">
  <div class="flex justify-center">
    <div class="join">
      <button
        class="join-item btn sm:text-lg"
        on:click={() => goto(toHourId(moment(data.pageHour.current).subtract({ days: 1 })))}
        >‹</button
      >
      <button class="join-item btn sm:text-lg">{data.pageHour.current.format("dddd LL")}</button>
      <button class="join-item btn sm:text-lg hidden lg:block">
        {data.pageHour.current.format("UTCZ")}
      </button>
      <button
        class="join-item btn sm:text-lg"
        disabled={moment(data.pageHour.current)
          .startOf("date")
          .isSameOrAfter(moment(data.pageHour.max).startOf("date"))}
        on:click={() =>
          goto(
            toHourId(moment.min(moment(data.pageHour.current).add({ days: 1 }), data.pageHour.max)),
          )}>›</button
      >
      <button
        class="join-item btn sm:text-lg"
        disabled={moment(data.pageHour.current)
          .startOf("hour")
          .isSameOrAfter(moment().startOf("hour"))}
        on:click={() => goto(toHourId(moment()))}>»</button
      >
    </div>
  </div>

  <div class="flex justify-center">
    <div class="join">
      <button
        class="join-item btn btn-sm lg:max-xl:btn-xs"
        on:click={() => goto(toHourId(moment(data.pageHour.current).subtract({ hours: 1 })))}
        >‹</button
      >
      {#each [...Array(24).keys()].map( (hour) => moment(data.pageHour.current).hour(hour), ) as buttonHour}
        <button
          class="join-item btn btn-sm lg:max-xl:btn-xs hidden lg:block"
          class:btn-active={buttonHour.isSame(data.pageHour.current)}
          disabled={buttonHour.isAfter(data.pageHour.max)}
          on:click={() => goto(toHourId(buttonHour))}
        >
          {buttonHour.hours().toString().padStart(2, "0")}
        </button>
      {/each}
      <button class="join-item btn btn-sm lg:max-xl:btn-xs block lg:hidden">
        {data.pageHour.current.format("HH:00")}
      </button>
      <button class="join-item btn btn-sm lg:max-xl:btn-xs block lg:hidden">
        {data.pageHour.current.format("UTCZ")}
      </button>
      <button
        class="join-item btn btn-sm lg:max-xl:btn-xs"
        disabled={data.pageHour.current.isSameOrAfter(data.pageHour.max)}
        on:click={() => goto(toHourId(moment(data.pageHour.current).add({ hours: 1 })))}>›</button
      >
    </div>
  </div>
</div>

<div class="overflow-x-auto my-4">
  <table class="table table-sm table-responsive">
    <thead>
      <tr>
        <th>Timestamp</th>
        <th>Artist</th>
        <th>Title</th>
        <th />
      </tr>
    </thead>
    <tbody>
      {#each data.content.plays as play}
        <tr class={play.track.is_song ? "" : "italic text-neutral-300 dark:text-neutral-600"}>
          <td class="whitespace-nowrap w-0 max-sm:font-bold">
            {moment(play.played_at).format("HH:mm:ss")}
          </td>
          <td>{play.track.artist}</td>
          <td>{play.track.title}</td>
          <td class="whitespace-nowrap w-0">
            <button
              class="btn btn-xs"
              on:click={async () =>
                await navigator.clipboard.writeText(`${play.track.artist} ${play.track.title}`)}
            >
              Copy
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
