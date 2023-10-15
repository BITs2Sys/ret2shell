<script lang="ts">
  import RxImage from '$lib/components/RxImage.svelte'
  import RxTag from '$lib/components/RxTag.svelte'
  import { i18n } from '$lib/i18n'
  import { Permission, permissionToString, type User } from '$lib/models/user'

  export let loading: boolean = false
  export let user: User
</script>

<div class="w-full flex flex-col">
  <div class="h-28 flex flex-row items-center p-6 border-b border-b-base-content/5">
    <div class="rounded-full overflow-clip w-12 h-12 mx-2 ring-4 ring-offset-base-100 ring-offset-4">
      {#if user.cover_path}
        <RxImage src={user.cover_path} {loading}></RxImage>
      {:else}
        <div class="w-full h-full flex flex-col justify-center items-center">
          <span class="text-3xl font-bold">{user.name.slice(0, 1)}</span>
        </div>
      {/if}
    </div>
    <div class="flex flex-col justify-center space-y-2 ml-6">
      <span class="font-bold text-xl">{user.name}</span>
      <span class="font-bold text-base opacity-60">0x{user.id.toString(16).padStart(6, '0')}</span>
    </div>
  </div>
  <div class="flex flex-col p-6">
    <h2 class="font-bold text-base opacity-60 flex flex-row space-x-2 items-center">
      <span class="icon-[fluent--mail-16-regular] w-5 h-5"></span>
      <span>
        {$i18n.t('account.email')}
      </span>
    </h2>
    <a class="hover:underline mt-2 px-2" href={`mailto:${user.email}`}>{user.email}</a>
    <h2 class="font-bold text-base opacity-60 flex flex-row space-x-2 items-center mt-6">
      <span class="icon-[fluent--shield-question-16-regular] w-5 h-5"></span>
      <span>
        {$i18n.t('account.status')}
      </span>
    </h2>
    <p class="mt-2 flex flex-row flex-wrap">
      {#if user.banned}
        <RxTag class="m-1" label={$i18n.t('account.banned')} level="error"></RxTag>
      {/if}
      {#if user.hidden}
        <RxTag class="m-1" label={$i18n.t('account.hidden')} level="warning"></RxTag>
      {/if}
      {#if !user.banned && !user.hidden}
        <RxTag class="m-1" label={$i18n.t('account.ok')} level="success"></RxTag>
      {/if}
    </p>
    <h2 class="font-bold text-base opacity-60 flex flex-row space-x-2 items-center mt-6">
      <span class="icon-[fluent--key-multiple-16-regular] w-5 h-5"></span>
      <span>
        {$i18n.t('account.permissions')}
      </span>
    </h2>
    <p class="mt-2 flex flex-wrap flex-row">
      {#each user.permissions.filter((p) => p !== Permission.Basic) as permission}
        <RxTag class="m-1" label={permissionToString(permission)}></RxTag>
      {/each}
    </p>
  </div>
</div>
