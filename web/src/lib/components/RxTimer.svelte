<script lang="ts">
  /// time in seconds
  export let time = 0

  $: hours = Math.floor(time / 3600)
  $: minutes = Math.floor((time / 60) % 60)
  $: seconds = Math.floor(time % 60)
</script>

<div class="flex flex-row items-center space-x-0">
  <span>{hours.toString().padStart(2, '0')}</span>
  <span class="opacity-60 animate-pulse">:</span>
  <span>{minutes.toString().padStart(2, '0')}</span>
  <span class="opacity-60 animate-pulse">:</span>
  <span class="text-primary">{seconds.toString().padStart(2, '0')}</span>
</div>
