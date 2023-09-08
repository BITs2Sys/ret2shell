<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxCalendar from '$lib/components/RxCalendar.svelte'
</script>

<svelte:head><title>{$platform.name}</title></svelte:head>

<div class="flex-1 relative">
  <div class="absolute h-full w-full overflow-scroll snap-mandatory snap-y">
    <section class="h-full min-h-full snap-center flex flex-col items-center justify-center">
      <div class="flex-1" />
      <h1 class="text-3xl font-semibold">
        &nbsp;&nbsp;[&nbsp;{$platform.name}&nbsp;]&nbsp;<span class="text-primary animate-ping">_</span>
      </h1>
      <a class="text-xl text-error mt-8" href={$platform.subjectUrl}>{$platform.subjectInfo}</a>
      <div class="flex-1" />
      <div class="flex items-center justify-center h-24">
        <p
          class="pt-3 pb-3 pr-5 pl-5 mb-3 ml-3 mr-3 backdrop-blur backdrop-brightness-100 rounded-box text-gray-500 shadow-sm border border-base-content/5 inline-flex justify-center items-center flex-wrap"
        >
          (C) 2022 - {new Date().getFullYear()}&nbsp;
          <a href={$platform.footerUrl} class="link">{$platform.footerInfo}</a>
          {#if !$platform.hideMaker}
            &nbsp;<span class="opacity-60">|</span>&nbsp; By&nbsp;
            <a href="https://github.com/ret2shell" class="link">
              {$i18n.t('about.maker')}
            </a>
          {/if}
          {#if $platform.record}
            &nbsp;<span class="opacity-60">|</span>&nbsp;
            <a href="https://beian.miit.gov.cn" class="link">{$platform.record}</a>
          {/if}
          <a class="btn btn-sm btn-ghost btn-square ml-2" href="/surprise/sakana">
            <span class="icon-[fluent--gift-16-regular] w-6 h-6 opacity-80" />
          </a>
        </p>
      </div>
    </section>
    <section class="h-full min-h-full snap-center flex flex-col p-3 md:p-6">
      <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
        <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
        <h1 class="text-2xl font-bold">{$i18n.t('calendar.title')}</h1>
        <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
      </div>
      <div class="flex flex-1 flex-col lg:flex-row lg:p-24 lg:pt-12 lg:pb-36 space-y-6 lg:space-y-0 lg:space-x-24">
        <RxCalendar class="flex-1" />
        <div class="flex-1">

        </div>
      </div>
    </section>
  </div>
</div>
