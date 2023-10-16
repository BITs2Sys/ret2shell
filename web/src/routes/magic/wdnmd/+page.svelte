<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import { i18n } from '$lib/i18n'
  import { onMount } from 'svelte'
  import { blur, fly } from 'svelte/transition'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import { platform } from '$lib/stores/platform'

  let exprs: Array<unknown> = Array.from({ length: 100 }, (_, i) => null)
  const availableExprs = [
    'null',
    'undefined',
    '0',
    '1',
    '-1',
    '""',
    '" "',
    '"0"',
    '"1"',
    '"-1"',
    'true',
    'false',
    '[]',
    '[[]]',
    'String\n("")',
    'Number\n(0)',
    'String\n("0")',
    'Number\n(1)',
    'String\n("1")',
    'Number\n(-1)',
    'String\n("-1")',
    'Boolean\n(true)',
    'Boolean\n(false)',
    '[1]',
    '[0]',
    '[-1]',
    'Infinity',
    '-Infinity',
    'Object()',
    'NaN',
  ]
  function randomize() {
    exprs = Array.from({ length: 100 }, (_, i) => availableExprs[Math.floor(Math.random() * availableExprs.length)])
  }

  onMount(() => {
    randomize()
  })

  let score = 0
  let combo = 0
  let operator = '=='
  let availableOperators = [
    { id: '==', label: '==' },
    { id: '===', label: '===' },
    { id: '>=', label: '>=' },
    { id: '<=', label: '<=' },
    { id: '>= && <=', label: '>= && <=' },
  ]
  let prevClickIndex: number | null = null

  let logs: { state: boolean; expr: string }[] = []

  function isNearIndex(index: number) {
    if (prevClickIndex === null) {
      return false
    }
    let row = Math.floor(index / 10)
    let col = index % 10
    let prevRow = Math.floor(prevClickIndex / 10)
    let prevCol = prevClickIndex % 10
    // console.log(row, col, prevRow, prevCol, Math.abs(row - prevRow) && Math.abs(col - prevCol))
    return Math.abs(row - prevRow) + Math.abs(col - prevCol) <= 1
  }

  function resetColRow(index: number) {
    // only reset the row and col of the clicked button
    let row = Math.floor(index / 10)
    let col = index % 10
    // randomize the full row and col
    for (let i = 0; i < 10; i++) {
      exprs[row * 10 + i] = availableExprs[Math.floor(Math.random() * availableExprs.length)]
      exprs[i * 10 + col] = availableExprs[Math.floor(Math.random() * availableExprs.length)]
    }
  }

  function clicked(index: number) {
    if (prevClickIndex === null) {
      prevClickIndex = index
      return
    }
    if (prevClickIndex === index) {
      prevClickIndex = null
      return
    }
    let fullExpr = ''
    if (operator === '>= && <=') {
      fullExpr = `${exprs[prevClickIndex]} >= ${exprs[index]} && ${exprs[prevClickIndex]} <= ${exprs[index]}`
    } else fullExpr = `${exprs[prevClickIndex]} ${operator} ${exprs[index]}`
    console.log(fullExpr)
    if (eval(fullExpr) === true) {
      score += combo + 1
      combo++
      resetColRow(prevClickIndex)
      resetColRow(index)
      logs = [...logs, { state: true, expr: fullExpr }]
    } else {
      combo = 0
      logs = [
        ...logs,
        {
          state: false,
          expr: fullExpr
            .replace(' == ', ' != ')
            .replace(' === ', ' !== ')
            .replace(' >= ', ' != ')
            .replace(' <= ', ' != '),
        },
      ]
    }
    prevClickIndex = null
    logBottomEl.scrollIntoView({ behavior: 'smooth' })
  }
  let logBottomEl: HTMLDivElement
  let containerWidth = 0
  let containerHeight = 0
  $: boxSize = Math.min(containerWidth, containerHeight) - 64
</script>

<svelte:head>
  <title>{$i18n.t('surprise.jswdnmd.title')} - {$platform.name}</title>
</svelte:head>
<div class="flex-1 flex flex-row">
  <div
    class="w-1/5 min-w-[24rem] max-w-[32rem] bg-neutral/20 backdrop-blur border-r border-r-base-content/10 flex flex-col"
  >
    <div
      class="h-16 flex flex-col items-center justify-center bg-neutral/10 backdrop-blur border-b border-b-base-content/5"
    >
      <h1 class="text-base font-bold text-center">{$i18n.t('surprise.jswdnmd.title')}</h1>
    </div>
    <div class="flex-1 flex flex-col p-4 space-y-2">
      <h2 class="font-bold opacity-60">OPERATOR:</h2>
      <div class="flex flex-row relative">
        <RxSelect name="operator" bind:value={operator} availableOptions={availableOperators}></RxSelect>
      </div>
      <h2 class="font-bold opacity-60 !mt-4">STATUS:</h2>
      <p class="text-lg flex flex-row items-center justify-center p-2 font-bold">
        <span class="opacity-80">SCORE:</span>
        &nbsp;
        <span class="text-primary">{score}</span>
        <span class="opacity-40">&nbsp;|&nbsp;</span>
        <span class="opacity-80">COMBO:</span>
        &nbsp;
        <span class="text-primary">{combo}</span>
      </p>
      <h2 class="font-bold opacity-60">LOGS:</h2>
      <div class="flex-1 overflow-hidden relative">
        <div class="absolute top-0 left-0 w-full h-full overflow-scroll">
          <ul class="flex flex-col space-y-1">
            {#each logs as log}
              <li
                class="p-1 px-2 flex flex-row space-x-2 items-center border-b border-b-base-content/10"
                transition:fly={{ x: 24, duration: 300 }}
              >
                {#if log.state}
                  <span class="icon-[fluent--checkmark-16-regular] text-success flex-shrink-0"></span>
                {:else}
                  <span class="icon-[fluent--dismiss-16-regular] text-warning flex-shrink-0"></span>
                {/if}
                &nbsp;
                <span class="opacity-80">{log.expr}</span>
              </li>
            {/each}
          </ul>
          <div class="h-16" bind:this={logBottomEl}></div>
        </div>
      </div>
    </div>
    <div class="h-16 flex flex-col items-center justify-center">
      <p class="opacity-60">
        {$i18n.t('surprise.source')}:
        <a class="hover:underline" href="https://js.wdn.md">js.wdn.md</a>
      </p>
    </div>
  </div>
  <div
    class="flex-1 flex flex-row items-center justify-center"
    bind:clientHeight={containerHeight}
    bind:clientWidth={containerWidth}
  >
    <div class="grid grid-cols-10 grid-rows-[10] gap-1" style={`width: ${boxSize}px; height: ${boxSize}px;`}>
      {#each exprs as item, index}
        <div class="relative aspect-square">
          <div class="absolute top-0 left-0 right-0 bottom-0">
            <RxButton
              class="w-full h-full p-0"
              on:click={() => clicked(index)}
              active={prevClickIndex === index}
              disabled={prevClickIndex !== null && prevClickIndex !== index && !isNearIndex(index)}
            >
              <p class="w-full break-words whitespace-break-spaces text-sm">
                {#key item}
                  <span in:blur={{ amount: 20, duration: 300 }}>{item}</span>
                {/key}
              </p>
            </RxButton>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
