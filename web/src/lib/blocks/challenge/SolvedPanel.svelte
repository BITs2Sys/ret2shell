<script lang="ts">
  let clazz = ''
  export { clazz as class }

  $: classes = `${clazz}`
</script>

<div class={classes}></div>
