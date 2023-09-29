<script lang="ts">
  export let id: string | undefined = undefined
  export let name: string | undefined = undefined
  export let label: string | undefined = undefined
  export let checked: boolean = false
</script>

<label class="label cursor-pointer space-x-4">
  <input {id} {name} type="checkbox" class="checkbox" {...$$restProps} bind:checked />
  <slot>
    <span class="text-base">{label}</span>
  </slot>
</label>
