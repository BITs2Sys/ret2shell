<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxCard from '$lib/components/RxCard.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
</script>

<svelte:head><title>{$i18n.t('admin.title')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col items-center justify-center space-y-16">
  <h1 class="text-3xl font-bold flex flex-row items-center space-x-4">
    <span class="icon-[fluent--emoji-32-filled] w-8 h-8"></span>
    <span>{$i18n.t('admin.welcome')}</span>
  </h1>
  <div class="flex flex-row flex-wrap self-center justify-center">
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/statistics">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--data-pie-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.statistics')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.statisticsIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/platform">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--home-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.platform')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.platformIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/games">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--flag-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.game')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.gameIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/announcements">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--megaphone-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.announcements')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.announcementsIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/calendar">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--calendar-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.calendar')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.calendarIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/wiki">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--book-open-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.wiki')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.wikiIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
    <RxCard class="w-72 m-4">
      <a class="flex flex-col w-full h-full space-y-4" href="/admin/users">
        <div class="flex flex-row items-center space-x-2">
          <span class="icon-[fluent--person-24-regular] w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold">{$i18n.t('admin.routes.users')}</h2>
        </div>
        <p class="text-base opacity-60 w-full break-words">{$i18n.t('admin.usersIntro')}</p>
        <div class="flex-1"></div>
      </a>
    </RxCard>
  </div>
</div>
