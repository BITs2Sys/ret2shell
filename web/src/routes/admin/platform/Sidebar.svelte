<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'

  const routes = [
    {
      name: $i18n.t('admin.basicInfoSettings'),
      icon: 'icon-[fluent--info-20-regular]',
      link: '/admin/platform/info',
    },
    {
      name: $i18n.t('admin.captchaSettings'),
      icon: 'icon-[fluent--beaker-20-regular]',
      link: '/admin/platform/captcha',
    },
    {
      name: $i18n.t('admin.emailSettings'),
      icon: 'icon-[fluent--mail-20-regular]',
      link: '/admin/platform/email',
    },
    {
      name: $i18n.t('admin.mediaSettings'),
      icon: 'icon-[fluent--image-20-regular]',
      link: '/admin/platform/media',
    },
    {
      name: $i18n.t('admin.pusherSettings'),
      icon: 'icon-[fluent--bot-20-regular]',
      link: '/admin/platform/pusher',
    },
  ]
</script>

{#each routes as item}
  <RxLink
    class="flex-nowrap overflow-hidden"
    href={item.link}
    ghost
    justify="start"
    exactlyMatched={item.link === $page.url.pathname}
  >
    <span class={`${item.icon} w-5 h-5 flex-shrink-0`} />
    <span class="text-ellipsis whitespace-nowrap overflow-hidden">{item.name}</span>
  </RxLink>
{/each}
