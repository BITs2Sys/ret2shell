<script lang="ts">
  import RxPaginator from '$lib/components/RxPaginator.svelte'
  import RxTag from '$lib/components/RxTag.svelte'

  interface DataEntry {
    [key: string]: string | number | boolean | null
  }
  interface ColumnType {
    [key: string]: 'plain' | 'number' | 'tag' | 'bool' | 'date' | 'hidden' | 'action'
  }
  export let dataEntries: DataEntry[] = []
  export let columntypes: ColumnType = {}
  export let page: number = 0
  export let total: number = 1
</script>

<div class="flex flex-col space-y-2">
  <table class="table rounded-box overflow-clip bg-base-content/5 backdrop-blur">
    <thead class="text-base bg-neutral/80">
      <tr>
        {#each Object.keys(columntypes) as key}
          {#if columntypes[key] != 'hidden'}
            <td>{key}</td>
          {/if}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each dataEntries as dataEntry}
        <tr>
          {#each Object.keys(dataEntry) as key}
            {#if columntypes[key] == 'plain'}
              <td title={dataEntry[key]?.toString()} class="relative overflow-x-clip">
                <div class="absolute left-4 right-4 top-0 bottom-0 flex flex-row items-center overflow-x-clip">
                  <div class="flex-1 min-w-0 whitespace-nowrap text-ellipsis overflow-x-clip">{dataEntry[key]}</div>
                </div>
              </td>
            {:else if columntypes[key] == 'number' && typeof dataEntry[key] === 'number'}
              <td class="w-0">
                <span>{dataEntry[key]}</span>
              </td>
            {:else if columntypes[key] == 'tag'}
              <td class="w-0 whitespace-nowrap">
                <div class="flex flex-row items-center justify-start">
                  <RxTag level="info" label={dataEntry[key]?.toString()} />
                </div>
              </td>
            {:else if columntypes[key] == 'date' && typeof dataEntry[key] === 'number'}
              <td class="w-0 whitespace-nowrap">
                <span>
                  {new Date(
                    //@ts-expect-error fuck you ts
                    (dataEntry[key] || 0) * 1000
                  ).toLocaleString()}
                </span>
              </td>
            {:else if columntypes[key] == 'bool' && typeof dataEntry[key] === 'boolean'}
              <td class="w-0 whitespace-nowrap">
                {#if dataEntry[key] === true}
                  <span class="icon-[fluent--checkmark-circle-16-regular] w-6 h-6 text-success" />
                {:else}
                  <span class="icon-[fluent--dismiss-circle-16-regular] w-6 h-6 text-warning" />
                {/if}
              </td>
            {:else if columntypes[key] == 'action'}
              // TODO: add action implement
              <td>
                <span>{dataEntry[key]}</span>
              </td>
            {/if}
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
  <div class="flex flex-row items-center justify-center w-full">
    <RxPaginator {page} {total} />
  </div>
</div>
