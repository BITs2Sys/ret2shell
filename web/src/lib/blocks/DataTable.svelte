<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import RxPaginator from '$lib/components/RxPaginator.svelte'
  import RxTag from '$lib/components/RxTag.svelte'
  import { i18n } from '$lib/i18n'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from './DataTable'
  import { blur } from 'svelte/transition'

  export let data: object[]
  export let colDef: DTColumnsDef
  export let page: number
  export let total: number
  export let loading: boolean = false
  let defaultBooleanIcon = {
    true: 'icon-[fluent--checkmark-circle-16-regular] text-success',
    false: 'icon-[fluent--dismiss-circle-16-regular] text-warning',
  }
  export let booleanIconsDef: {
    [key: string]: IconDef
  } = {}

  interface IconDef {
    true: string
    false: string
  }
  let clazz = ''
  export { clazz as class }
  $: classes = `flex flex-col justify-between ${clazz}`
  // level: text-info text-success text-warning text-error
  export let actions: DTColumnAction[] = []
  $: types = Object.keys(colDef)
    .map((key) => {
      return {
        [key]: colDef[key].type,
      }
    })
    .reduce((a, b) => ({ ...a, ...b }), {})
  $: dataEntries = data as DTDataEntry[]

  function renderLink(pattern: string, data: DTDataEntry) {
    // `pattern` will be any str with `{key}` in it,
    // we should replace it with the value of `data[key]`
    return pattern.replace(/{(\w+)}/g, (_, key) => data[key]?.toString() || '')
  }
</script>

<div class={classes}>
  <table class="w-full max-w-full table table-auto rounded-box overflow-hidden bg-base-content/5 backdrop-blur">
    <thead class="text-base bg-neutral/80">
      <tr class="border-b-base-100/60">
        {#each Object.keys(colDef) as key}
          {#if types[key] != 'hidden'}
            <td
              class={colDef[key].justify && colDef[key].justify}
              style={`${typeof colDef[key].sizePolicy === 'number' ? `width: ${colDef[key].sizePolicy}px` : ''}`}
            >
              {colDef[key].header}
            </td>
          {/if}
        {/each}
        {#if actions.length > 0}
          <td class="w-0"></td>
        {/if}
      </tr>
    </thead>
    <tbody class="w-full max-w-full overflow-hidden text-base relative">
      {#if loading}
        <div
          class="absolute top-0 left-0 w-full h-full flex flex-row justify-center items-center bg-neutral z-20"
          transition:blur={{ amount: 20, duration: 300 }}
        >
          <span class="loading loading-spinner loading-sm" />
        </div>
      {/if}
      {#if dataEntries.length === 0}
        <tr class="border-b-base-100/60">
          <td colspan={Object.keys(colDef).length + (actions.length > 0 ? 1 : 0)} class="text-center">
            <span class="text-base opacity-80">{$i18n.t('table.noData')}</span>
          </td>
        </tr>
      {/if}
      {#each dataEntries as dataEntry}
        <tr class="border-b-base-100/60">
          {#each Object.keys(dataEntry) as key}
            {#if types[key] == 'plain'}
              <td
                title={dataEntry[key]?.toString().includes('|')
                  ? decodeURI(dataEntry[key]?.toString().split('|')[0] || '')
                  : decodeURI(dataEntry[key]?.toString() || '')}
                class={`truncate ${colDef[key].sizePolicy === 'shrink' ? 'w-0' : 'w-full max-w-0'} ${
                  colDef[key].justify && colDef[key].justify
                } ${colDef[key].dimmed && 'opacity-60'}`}
              >
                {#if dataEntry[key]?.toString().includes('|')}
                  <a class="hover:underline w-full" href={dataEntry[key]?.toString().split('|')[1]}>
                    {decodeURI(dataEntry[key]?.toString().split('|')[0] || '')}
                  </a>
                {:else}
                  <span>{dataEntry[key]?.toString()}</span>
                {/if}
              </td>
            {:else if types[key] == 'number' && typeof dataEntry[key] === 'number'}
              <td
                class={`${colDef[key].sizePolicy === 'shrink' && 'w-0'} ${colDef[key].dimmed && 'opacity-60'} ${
                  colDef[key].justify && colDef[key].justify
                }`}
              >
                <span>{dataEntry[key]}</span>
              </td>
            {:else if types[key] == 'tag'}
              <td
                class={`whitespace-nowrap ${colDef[key].sizePolicy === 'shrink' && 'w-0'} ${
                  colDef[key].dimmed && 'opacity-60'
                } ${colDef[key].justify && colDef[key].justify}`}
              >
                <div class="flex flex-row items-center justify-start">
                  <RxTag level="info" label={dataEntry[key]?.toString()} />
                </div>
              </td>
            {:else if types[key] == 'tags'}
              <td
                class={`whitespace-nowrap py-0 ${colDef[key].sizePolicy === 'shrink' && 'w-0'} ${
                  colDef[key].dimmed && 'opacity-60'
                } ${colDef[key].justify && colDef[key].justify}`}
              >
                <div class="flex flex-row space-x-2 items-center justify-start">
                  <!-- render three tags, and display a `...` if have more -->
                  {#each (dataEntry[key]?.toString().split('|') || []).slice(0, 2) as tag}
                    <RxTag level="info" label={tag} />
                  {/each}
                  {#if (dataEntry[key]?.toString().split('|') || []).length > 2}
                    <RxTag
                      level="info"
                      label="..."
                      title={(dataEntry[key]?.toString().split('|') || []).slice(2).join(', ')}
                    />
                  {/if}
                </div>
              </td>
            {:else if types[key] == 'date' && typeof dataEntry[key] === 'number'}
              <td
                class={`whitespace-nowrap ${colDef[key].sizePolicy === 'shrink' && 'w-0'} ${
                  colDef[key].dimmed && 'opacity-60'
                } ${colDef[key].justify && colDef[key].justify}`}
              >
                <span>
                  {new Date(
                    //@ts-expect-error fuck you ts
                    (dataEntry[key] || 0) * 1000
                  ).toLocaleString()}
                </span>
              </td>
            {:else if types[key] == 'bool' && typeof dataEntry[key] === 'boolean'}
              <td
                class={`whitespace-nowrap leading-[0] ${colDef[key].sizePolicy === 'shrink' && 'w-0'} ${
                  colDef[key].dimmed && 'opacity-60'
                } ${colDef[key].justify && colDef[key].justify}`}
              >
                {#if dataEntry[key] === true}
                  <span
                    class={`w-5 h-5 ${
                      Object.keys(booleanIconsDef).find((bkey) => bkey === key)
                        ? booleanIconsDef[
                            Object.keys(booleanIconsDef)
                              .find((bkey) => bkey === key)
                              ?.toString() || ''
                          ]?.true
                        : defaultBooleanIcon.true
                    }`}
                  />
                {:else}
                  <span
                    class={`w-5 h-5 ${
                      Object.keys(booleanIconsDef).find((bkey) => bkey === key)
                        ? booleanIconsDef[
                            Object.keys(booleanIconsDef)
                              .find((bkey) => bkey === key)
                              ?.toString() || ''
                          ]?.false
                        : defaultBooleanIcon.false
                    }`}
                  />
                {/if}
              </td>
            {/if}
          {/each}
          {#if actions.length > 0}
            <td class="w-0 whitespace-nowrap py-0">
              <div class="flex flex-row space-x-2">
                {#each actions as item}
                  {#if item.type === 'button'}
                    <RxButton
                      size="sm"
                      ghost
                      square={item.label.length === 0}
                      on:click={() => {
                        item.onClick && item.onClick(dataEntry)
                      }}
                    >
                      <span class={`${item.icon} w-4 h-4 text-${item.level}`}></span>
                      {#if item.label.length > 0}
                        <span class="text-sm">{item.label}</span>
                      {/if}
                    </RxButton>
                  {:else if item.type === 'link'}
                    <RxLink
                      size="sm"
                      ghost
                      square={item.label.length === 0}
                      href={renderLink(item.href || '', dataEntry)}
                    >
                      <span class={`${item.icon} w-4 h-4 text-${item.level}`}></span>
                      {#if item.label.length > 0}
                        <span class="text-sm">{item.label}</span>
                      {/if}
                    </RxLink>
                  {/if}
                {/each}
              </div>
            </td>
          {/if}
        </tr>
      {/each}
    </tbody>
  </table>
  <div class="flex flex-row items-center justify-center w-full">
    <RxPaginator bind:page {total} />
  </div>
</div>
