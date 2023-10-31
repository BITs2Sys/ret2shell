<script lang="ts">
  export let icon: string = 'icon-[fluent--info-24-filled] w-6 h-6'
  export let title: string = ''
  export let value: string | number = ''
</script>

<div class="flex-1 stat rounded-box bg-neutral/20 backdrop-blur border border-base-content/5">
  <div class="stat-figure">
    <span class={icon}></span>
  </div>
  <div class="stat-title">
    <slot name="title">
      {title}
    </slot>
  </div>
  <div class="stat-value">
    <slot>
      {value.toString()}
    </slot>
  </div>
  <div class="stat-desc">
    <slot name="desc" />
  </div>
</div>
