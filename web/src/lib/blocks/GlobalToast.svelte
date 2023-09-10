<script lang="ts">
  import { toast } from '$lib/stores/toast'
  import RxToast from '$lib/components/RxToast.svelte'
  import { flip } from 'svelte/animate'
  import { quintOut } from 'svelte/easing'
</script>

<div class="fixed bottom-0 right-0 max-w-[50vw] flex flex-col items-end p-4 space-y-4">
  {#each $toast.toasts as toast (toast)}
    <div animate:flip={{ delay: 100, duration: 300, easing: quintOut }}>
      <RxToast
        id={toast.id}
        message={toast.message}
        level={toast.level}
        persistTime={toast.persistTime}
        acceptMessage={toast.acceptMessage}
        rejectMessage={toast.rejectMessage}
        on:accept={toast.accept}
        on:reject={toast.reject}
      />
    </div>
  {/each}
</div>
