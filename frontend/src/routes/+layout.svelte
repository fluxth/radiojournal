<script lang="ts">
  import type { Snippet } from "svelte";

  import "nprogress/nprogress.css";
  import "../app.css";

  import NProgress from "nprogress";
  import { navigating } from "$app/state";
  import { resolve } from "$app/paths";

  type Props = {
    children?: Snippet;
  };

  let { children }: Props = $props();

  NProgress.configure({ minimum: 0.16, showSpinner: false });

  $effect(() => {
    if (navigating.to) NProgress.start();
    else NProgress.done();
  });
</script>

<div class="navbar bg-base-100 shadow-lg">
  <div class="container mx-auto px-4">
    <a class="btn btn-ghost text-xl" href={resolve("/")}>radiojournal</a>
  </div>
</div>

<div class="container mx-auto px-4">
  {@render children?.()}
</div>
