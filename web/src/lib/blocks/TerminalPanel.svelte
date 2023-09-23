<script lang="ts">
  let clazz = ''
  export { clazz as class }
  $: classes = `flex-1 relative ${clazz}`
  let terminal: HTMLDivElement
</script>

<div class={classes}>
  <div class="absolute top-4 left-4 right-4 bottom-4" bind:this={terminal}></div>
</div>
