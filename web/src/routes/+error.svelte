<script lang="ts">
    import { page } from '$app/stores'
</script>

<div class="flex-1 flex flex-col items-center justify-center">
    <h1 class="font-bold text-6xl">{$page.status}</h1>
    <p class="text-xl mt-8">{$page.error.message}</p>
</div>
