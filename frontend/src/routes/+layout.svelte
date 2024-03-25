<script>
  import "nprogress/nprogress.css";
  import "../app.scss";

  import NProgress from "nprogress";
  import { navigating } from "$app/stores";

  NProgress.configure({ minimum: 0.16, showSpinner: false });

  $: if ($navigating) NProgress.start();
  $: if (!$navigating) NProgress.done();
</script>

<div class="navbar bg-base-100 shadow-lg">
  <div class="container mx-auto px-4">
    <a class="btn btn-ghost text-xl" href="/">radiojournal</a>
  </div>
</div>

<div class="container mx-auto px-4">
  <slot />
</div>
