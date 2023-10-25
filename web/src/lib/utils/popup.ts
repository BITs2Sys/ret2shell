import { computePosition, autoUpdate, offset, shift, flip } from '@floating-ui/dom'

/** Placement https://floating-ui.com/docs/computePosition#placement */
export type Direction = 'top' | 'bottom' | 'left' | 'right'
export type Placement = Direction | `${Direction}-start` | `${Direction}-end`

// Options & Middleware
export interface Middleware {
  // Required ---
  /** Offset middleware settings: https://floating-ui.com/docs/offset */
  offset?: number | Record<string, unknown>
  /** Shift middleware settings: https://floating-ui.com/docs/shift */
  shift?: Record<string, unknown>
  /** Flip middleware settings: https://floating-ui.com/docs/flip */
  flip?: Record<string, unknown>
  // Optional ---
  /** Size middleware settings: https://floating-ui.com/docs/size */
  size?: Record<string, unknown>
  /** Auto Placement middleware settings: https://floating-ui.com/docs/autoPlacement */
  autoPlacement?: Record<string, unknown>
  /** Hide middleware settings: https://floating-ui.com/docs/hide */
  hide?: Record<string, unknown>
  /** Inline middleware settings: https://floating-ui.com/docs/inline */
  inline?: Record<string, unknown>
}

export interface PopupSettings {
  /** Provide the event type. */
  event: 'click' | 'hover' | 'focus-blur' | 'focus-click' | 'click-blur'
  /** Match the popup data value `data-popup="targetNameHere"` */
  target: string
  /** Set the placement position. Defaults 'bottom'. */
  placement?: Placement
  /** Query elements that close the popup when clicked. Defaults `'a[href], button'`. */
  closeQuery?: string
  /** Optional callback function that reports state change. */
  state?: (event: { state: boolean }) => void
  /** Provide Floating UI middleware settings. */
  middleware?: Middleware
}

export function popup(triggerNode: HTMLElement, args: PopupSettings) {
  // Local State
  const popupState = {
    open: false,
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    autoUpdateCleanup: () => {},
  }
  const focusableAllowedList = ':is(a[href], button, input, textarea, select, details, [tabindex]):not([tabindex="-1"])'
  let focusablePopupElements: HTMLElement[]
  // Elements
  let elemPopup: HTMLElement

  function setDomElements(): void {
    elemPopup = document.querySelector(`[data-popup="${args.target}"]`) ?? document.createElement('div')
    Object.assign(elemPopup.style, {
      position: 'absolute',
      display: 'none',
    })
  }
  setDomElements() // init

  // Render Floating UI Popup
  function render(): void {
    // Error handling for required Floating UI modules
    if (!elemPopup) throw new Error(`The data-popup="${args.target}" element was not found.`)
    if (!computePosition) throw new Error(`Floating UI 'computePosition' not found for data-popup="${args.target}".`)
    if (!offset) throw new Error(`Floating UI 'offset' not found for data-popup="${args.target}".`)
    if (!shift) throw new Error(`Floating UI 'shift' not found for data-popup="${args.target}".`)
    if (!flip) throw new Error(`Floating UI 'flip' not found for data-popup="${args.target}".`)

    // Floating UI Compute Position
    // https://floating-ui.com/docs/computePosition
    computePosition(triggerNode, elemPopup, {
      placement: args.placement ?? 'bottom',

      // Middleware - NOTE: the order matters:
      // https://floating-ui.com/docs/middleware#ordering
      middleware: [
        // https://floating-ui.com/docs/offset
        offset(args.middleware?.offset ?? 8),
        // https://floating-ui.com/docs/shift
        shift(args.middleware?.shift ?? { padding: 8 }),
        // https://floating-ui.com/docs/flip
        flip(args.middleware?.flip),
      ],
    }).then(({ x, y }) => {
      Object.assign(elemPopup.style, {
        left: `${x}px`,
        top: `${y}px`,
      })
    })
  }

  // State Handlers
  function open(): void {
    if (!elemPopup) return
    // Set open state to on
    popupState.open = true
    // Return the current state
    if (args.state) args.state({ state: popupState.open })
    // Update render settings
    render()
    // Update the DOM
    elemPopup.style.display = 'block'
    elemPopup.style.pointerEvents = 'auto'
    // Trigger Floating UI autoUpdate (open only)
    // https://floating-ui.com/docs/autoUpdate
    popupState.autoUpdateCleanup = autoUpdate(triggerNode, elemPopup, render)
    // Focus the first focusable element within the popup
    focusablePopupElements = Array.from(elemPopup?.querySelectorAll(focusableAllowedList))
  }
  function close(callback?: () => void): void {
    if (!elemPopup) return
    // Set open state to off
    popupState.open = false
    // Return the current state
    if (args.state) args.state({ state: popupState.open })
    // Update the DOM
    elemPopup.style.display = 'none'
    elemPopup.style.pointerEvents = 'none'
    // Cleanup Floating UI autoUpdate (close only)
    if (popupState.autoUpdateCleanup) popupState.autoUpdateCleanup()
    // Trigger callback
    if (callback) callback()
  }

  // Event Handlers
  function toggle(): void {
    !popupState.open ? open() : close()
  }
  function onWindowClick(event: Event): void {
    // Return if the popup is not yet open
    if (!popupState.open) return
    // Return if click is the trigger element
    if (triggerNode.contains(event.target as Node)) return
    // If click it outside the popup
    if (elemPopup && !elemPopup.contains(event.target as Node)) {
      close()
      return
    }
    if (args.event === 'click-blur' || args.event === 'focus-click') {
      // do not close window
      return
    }
    // Handle Close Query State
    const closeQueryString: string = args.closeQuery === undefined ? 'a[href], button' : args.closeQuery
    const closableMenuElements = elemPopup?.querySelectorAll(closeQueryString)
    closableMenuElements?.forEach((elem) => {
      if (elem.contains(event.target as Node)) close()
    })
  }

  // Keyboard Interactions for A11y
  const onWindowKeyDown = (event: KeyboardEvent): void => {
    if (!popupState.open) return
    // Handle keys
    const key: string = event.key
    // On Esc key
    if (key === 'Escape') {
      event.preventDefault()
      triggerNode.focus()
      close()
      return
    }
    // On Tab or ArrowDown key
    const triggerMenuFocused: boolean = popupState.open && document.activeElement === triggerNode
    if (
      triggerMenuFocused &&
      (key === 'ArrowDown' || key === 'Tab') &&
      focusableAllowedList.length > 0 &&
      focusablePopupElements.length > 0
    ) {
      event.preventDefault()
      focusablePopupElements[0].focus()
    }
  }

  // Event Listeners
  switch (args.event) {
    case 'click':
      triggerNode.addEventListener('click', toggle, true)
      window.addEventListener('click', onWindowClick, true)
      break
    case 'hover':
      triggerNode.addEventListener('mouseover', open, true)
      triggerNode.addEventListener('mouseleave', () => close(), true)
      break
    case 'click-blur':
      triggerNode.addEventListener('click', open, true)
      window.addEventListener('click', onWindowClick, true)
      break
    case 'focus-blur':
      triggerNode.addEventListener('focus', toggle, true)
      triggerNode.addEventListener('blur', () => close(), true)
      break
    case 'focus-click':
      triggerNode.addEventListener('focus', open, true)
      window.addEventListener('click', onWindowClick, true)
      break
    default:
      throw new Error(`Event value of '${args.event}' is not supported.`)
  }
  window.addEventListener('keydown', onWindowKeyDown, true)

  // Render popup on initialization
  render()

  // Lifecycle
  return {
    update(newArgs: PopupSettings) {
      close(() => {
        args = newArgs
        render()
        setDomElements()
      })
    },
    destroy() {
      // Trigger Events
      triggerNode.removeEventListener('click', toggle, true)
      triggerNode.removeEventListener('mouseover', open, true)
      triggerNode.removeEventListener('mouseleave', () => close(), true)
      triggerNode.removeEventListener('focus', toggle, true)
      triggerNode.removeEventListener('focus', open, true)
      triggerNode.removeEventListener('blur', () => close(), true)
      // Window Events
      window.removeEventListener('click', onWindowClick, true)
      window.removeEventListener('keydown', onWindowKeyDown, true)
    },
  }
}
