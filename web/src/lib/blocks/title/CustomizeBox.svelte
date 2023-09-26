<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { theme } from '$lib/stores/theme'
  import RxButton from '$lib/components/RxButton.svelte'

  function toggleColorScheme(scheme: string) {
    $theme.colorScheme = scheme
  }

  function toggleLanguage(language: string) {
    $i18n.changeLanguage(language)
  }
</script>

<div class="p-2 flex flex-col">
  <div class="text-sm opacity-60 p-1">{$i18n.t('theme.colorScheme')}</div>
  <div class="flex flex-row space-x-2">
    <RxButton ghost class="flex-1" on:click={() => toggleColorScheme('light')}>
      <span
        class="icon-[fluent--weather-sunny-16-regular] w-6 h-6"
        class:text-primary={$theme.colorScheme === 'light'}
      />
    </RxButton>
    <RxButton ghost class="flex-1" on:click={() => toggleColorScheme('dark')}>
      <span class="icon-[fluent--weather-moon-16-regular] w-6 h-6" class:text-primary={$theme.colorScheme === 'dark'} />
    </RxButton>
  </div>
  <div class="text-sm opacity-60 p-2">{$i18n.t('theme.language')}</div>
  <div class="flex flex-col">
    <RxButton
      ghost
      justify="start"
      class="flex-1"
      active={$i18n.language === 'zh_CN'}
      on:click={() => toggleLanguage('zh_CN')}
    >
      <span class="icon-[fluent--local-language-16-regular] w-6 h-6" />
      <span>{$i18n.t('theme.simplifiedChinese')}</span>
    </RxButton>
    <RxButton
      ghost
      justify="start"
      class="flex-1"
      active={$i18n.language === 'en_US'}
      on:click={() => toggleLanguage('en_US')}
    >
      <span class="icon-[fluent--local-language-16-regular] w-6 h-6" />
      <span>{$i18n.t('theme.english')}</span>
    </RxButton>
  </div>
</div>
