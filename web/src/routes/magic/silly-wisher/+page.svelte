<script lang="ts">
  import { page } from '$app/stores'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCard from '$lib/components/RxCard.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'

  let wishTypeArray = {
    'limited-role': $i18n.t('surprise.sillywisher.limitedRoleWish'),
    'limited-weapon': $i18n.t('surprise.sillywisher.limitedWeaponWish'),
    resident: $i18n.t('surprise.sillywisher.residentWish'),
  }

  let wishType: 'limited-role' | 'limited-weapon' | 'resident' = 'limited-role'

  $: wishTypeName = wishTypeArray[wishType]

  page.subscribe((value) => {
    if (value.url.hash) {
      wishType = value.url.hash.slice(1) as 'limited-role' | 'limited-weapon' | 'resident'
    }
  })
</script>

<div class="flex-1 flex flex-col">
  <div class="h-20 flex flex-row items-center space-x-4 pl-4 pr-24">
    <div>
      <RxLink href="#limited-role">{$i18n.t('surprise.sillywisher.limitedRoleWish')}</RxLink>
      <RxLink href="#limited-weapon">{$i18n.t('surprise.sillywisher.limitedWeaponWish')}</RxLink>
      <RxLink href="#resident">{$i18n.t('surprise.sillywisher.residentWish')}</RxLink>
    </div>
    <div class="flex-1"></div>
    <div class="join">
      <RxButton class="join-item">
        <span class="icon-[fluent--sparkle-20-filled] w-5 h-5"></span>
        <span>1600</span>
      </RxButton>
      <RxButton class="join-item ml-0">
        <span class="icon-[fluent--add-20-regular] w-5 h-5"></span>
      </RxButton>
    </div>
    <RxButton>
      <span class="icon-[fluent--sport-baseball-20-filled] w-5 h-5"></span>
      <span>70</span>
    </RxButton>
  </div>
  <div class="flex-1 flex py-12 px-48">
    <RxCard class="flex-1 relative flex flex-row">
      <div class="w-2/5 flex flex-col p-16 px-24 space-y-4">
        <h1 class="text-7xl font-bold flex flex-wrap">
          <span class="text-primary">{$i18n.t('surprise.sillywisher.malicious')}</span>
          <span>{$i18n.t('surprise.sillywisher.payload')}</span>
        </h1>
        <p class="text-lg">{$i18n.t('surprise.sillywisher.opened')}</p>
        <div class="h-[2px] bg-base-content/10"></div>
        <div class="flex">
          <p class="text-lg py-1 px-3 bg-primary text-white font-bold">
            {wishTypeName}
          </p>
        </div>
      </div>
    </RxCard>
  </div>
  <div class="h-24 flex flex-row space-x-8 px-24">
    <RxButton size="lg">{$i18n.t('surprise.sillywisher.detail')}</RxButton>
    <RxButton size="lg">{$i18n.t('surprise.sillywisher.history')}</RxButton>
    <div class="flex-1"></div>
    <RxButton size="lg" title={$i18n.t('surprise.sillywisher.justSuoha10Times')}>
      <span class="icon-[fluent--sparkle-20-filled] w-5 h-5"></span>
      <span>
        {$i18n.t('surprise.sillywisher.wishonce')}
      </span>
    </RxButton>
    <RxButton size="lg" title={$i18n.t('surprise.sillywisher.justSuoha')}>
      <span class="icon-[fluent--sparkle-20-filled] w-5 h-5"></span>
      <span>
        {$i18n.t('surprise.sillywisher.wish10s')}
      </span>
    </RxButton>
  </div>
</div>
