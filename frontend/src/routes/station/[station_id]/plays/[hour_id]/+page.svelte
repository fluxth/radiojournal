<script lang="ts">
  import type { PageData } from "./$types";
  import type { Dayjs } from "dayjs";

  import { goto } from "$app/navigation";
  import { toHourId } from "$lib/helpers";
  import dayjs from "$lib/dayjs";

  import windowsZones from "$lib/data/windowsZones.json";

  export let data: PageData;

  const TIMEZONE_LOCALSTORAGE_KEY = "timezone";

  const isValidTimezoneOrNull = (timezone: string | null): string | null => {
    if (timezone && !Intl.supportedValuesOf("timeZone").includes(timezone)) return null;
    return timezone;
  };

  let currentTimezone: string | null = isValidTimezoneOrNull(
    localStorage.getItem(TIMEZONE_LOCALSTORAGE_KEY),
  );

  $: currentPageHour = currentTimezone
    ? data.pageHour.current.tz(currentTimezone)
    : data.pageHour.current;
  $: maxPageHour = currentTimezone ? data.pageHour.max.tz(currentTimezone) : data.pageHour.max;

  let timezoneModal: HTMLDialogElement;

  $: timezones = windowsZones
    .map((zone, index) => {
      const zoneHour = data.pageHour.current.tz(zone.id);
      return {
        ...zone,
        index,
        offset: zoneHour.utcOffset(),
        offsetString: `UTC${zoneHour.format("Z")}`,
      };
    })
    .sort((a, b) => a.offset - b.offset || a.index - b.index);

  const getHoursOfCurrentDay = (day: Dayjs) => {
    const endHour = day.endOf("day");
    const hours = [];
    for (
      let startHour = day.startOf("day");
      !startHour.utc().isAfter(endHour.utc());
      startHour = startHour.add(dayjs.duration({ hours: 1 }))
    )
      hours.push(startHour);

    return hours;
  };

  const refresh = async () => {
    await data.invalidate();
  };

  const saveTimezone = (timezone: string | null) => {
    if (timezone) localStorage.setItem(TIMEZONE_LOCALSTORAGE_KEY, timezone);
    else localStorage.removeItem(TIMEZONE_LOCALSTORAGE_KEY);
  };

  const gotoYesterday = async () =>
    await goto(toHourId(currentPageHour.subtract(dayjs.duration({ days: 1 }))));

  const gotoTomorrow = async () =>
    await goto(
      toHourId(dayjs.min(currentPageHour.add(dayjs.duration({ days: 1 })), maxPageHour) as Dayjs),
    );

  const gotoNow = async () => await goto(toHourId(dayjs()));
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
    <li><a href={`/station/${data.station.id}/plays`}>{data.station.name}</a></li>
    <li>Play History</li>
  </ul>
</div>

<div class="my-4 flex flex-col items-center gap-1">
  <div class="flex justify-center">
    <div class="join">
      <button class="join-item btn sm:text-lg" on:click={gotoYesterday}>‹</button>
      <button class="join-item btn sm:text-lg">{currentPageHour.format("dddd, LL")}</button>
      <button
        class="join-item btn sm:text-lg hidden lg:block"
        on:click={() => timezoneModal.showModal()}
      >
        {currentPageHour.format("UTCZ")}
      </button>
      <button
        class="join-item btn sm:text-lg"
        disabled={currentPageHour.startOf("date").isSameOrAfter(maxPageHour.startOf("date"))}
        on:click={gotoTomorrow}>›</button
      >
      <button
        class="join-item btn sm:text-lg"
        disabled={currentPageHour.startOf("hour").isSameOrAfter(dayjs().startOf("hour"))}
        on:click={gotoNow}>»</button
      >
    </div>
  </div>

  <div class="flex justify-center">
    <div class="join">
      <button
        class="join-item btn btn-sm lg:max-xl:btn-xs"
        on:click={() => goto(toHourId(currentPageHour.subtract(dayjs.duration({ hours: 1 }))))}
        >‹</button
      >
      {#each getHoursOfCurrentDay(currentPageHour) as buttonHour}
        <button
          class="join-item btn btn-sm lg:max-xl:btn-xs hidden lg:block"
          class:btn-active={currentPageHour.isSame(buttonHour)}
          disabled={currentPageHour.isSameOrAfter(maxPageHour.startOf("date")) &&
            buttonHour.isAfter(maxPageHour)}
          on:click={() => goto(toHourId(buttonHour))}
        >
          {buttonHour.hour().toString().padStart(2, "0")}
        </button>
      {/each}
      <button class="join-item btn btn-sm lg:max-xl:btn-xs block lg:hidden">
        {currentPageHour.format("HH:00")}
      </button>
      <button
        class="join-item btn btn-sm lg:max-xl:btn-xs block lg:hidden"
        on:click={() => timezoneModal.showModal()}
      >
        {currentPageHour.format("UTCZ")}
      </button>
      <button
        class="join-item btn btn-sm lg:max-xl:btn-xs"
        disabled={currentPageHour.isSameOrAfter(maxPageHour)}
        on:click={() => goto(toHourId(currentPageHour.add(dayjs.duration({ hours: 1 }))))}>›</button
      >
    </div>
  </div>
</div>

<div class="overflow-x-auto my-4">
  <table class="table table-sm table-responsive table-fixed">
    <thead>
      <tr>
        <th class="w-24">Timestamp</th>
        <th>Artist</th>
        <th>Title</th>
        <th class="w-24" />
      </tr>
    </thead>
    <tbody>
      {#each data.content.plays as play}
        <tr class={play.track.is_song ? "" : "italic text-neutral-300 dark:text-neutral-600"}>
          <td class="max-sm:font-bold">
            {(currentTimezone
              ? dayjs(play.played_at).tz(currentTimezone)
              : dayjs(play.played_at)
            ).format("HH:mm:ss")}
          </td>
          <td>
            <a
              class="link"
              href={`/station/${data.station.id}/artist/${encodeURIComponent(play.track.artist)}`}
            >
              {play.track.artist}
            </a>
          </td>
          <td>
            <a class="link" href="/station/{data.station.id}/track/{play.track.id}">
              {play.track.title}
            </a>
          </td>
          <td class="sm:text-right">
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

<dialog class="modal" bind:this={timezoneModal}>
  <div class="modal-box">
    <form method="dialog">
      <button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2">✕</button>
    </form>
    <h3 class="font-bold text-lg -mt-2 mb-4">Timezone</h3>
    <div>
      <select
        class="select select-bordered w-full"
        bind:value={currentTimezone}
        on:change={() => saveTimezone(currentTimezone)}
      >
        {#if currentTimezone !== null && timezones.find((zone) => zone.id === currentTimezone) === undefined}
          <option disabled value={currentTimezone}>{currentTimezone}</option>
        {/if}
        <option value={null}>System Default</option>
        {#each timezones as timezone}
          <option value={timezone.id}>
            ({timezone.offsetString}) {timezone.label}
          </option>
        {/each}
      </select>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>
