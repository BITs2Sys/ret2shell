import { Checkbox, type CheckboxRootProps, Popover } from "@ark-ui/solid";
import { createMemo, type JSX, splitProps } from "solid-js";
import { Portal } from "solid-js/web";

export type CheckboxProps = {
  ghost?: boolean;
  error?: string;
  uncheckedIcon?: string;
  checkedIcon?: string;
  inputProps?: JSX.IntrinsicElements["input"];
};

export default function (
  props: CheckboxProps &
    CheckboxRootProps & {
      children?: JSX.Element;
    }
) {
  const [checkboxProps, rest] = splitProps(props, ["ghost", "error", "inputProps", "uncheckedIcon", "checkedIcon"]);

  const classes = createMemo(() =>
    `btn btn-md items-center justify-center btn-square ${checkboxProps.ghost ? "btn-ghost" : ""} ${rest.disabled ? "btn-disabled" : ""} ${rest.class}`.trim()
  );
  const iconClasses = createMemo(
    () =>
      `data-[state=unchecked]:${checkboxProps.uncheckedIcon} data-[state=checked]:${checkboxProps.checkedIcon} data-[state=checked]:text-primary !w-5 !h-5`
  );

  // data-[state=unchecked]:icon-[fluent--chat-20-regular] data-[state=checked]:icon-[fluent--chat-20-filled]
  // data-[state=unchecked]:icon-[fluent--pin-20-regular] data-[state=checked]:icon-[fluent--pin-20-filled]
  // data-[state=unchecked]:icon-[fluent--checkmark-20-regular] data-[state=checked]:icon-[fluent--checkmark-20-filled]
  // data-[state=unchecked]:icon-[fluent--checkmark-circle-20-regular] data-[state=checked]:icon-[fluent--checkmark-circle-20-filled]
  // data-[state=unchecked]:icon-[fluent--accessibility-checkmark-20-regular] data-[state=checked]:icon-[fluent--accessibility-checkmark-20-filled]
  // data-[state=unchecked]:icon-[fluent--wifi-off-20-regular] data-[state=checked]:icon-[fluent--wifi-off-20-filled]
  // data-[state=unchecked]:icon-[fluent--people-audience-20-regular] data-[state=checked]:icon-[fluent--people-audience-20-filled]
  // data-[state=unchecked]:icon-[fluent--edit-20-regular] data-[state=checked]:icon-[fluent--edit-20-filled]
  // data-[state=unchecked]:icon-[fluent--megaphone-loud-20-regular] data-[state=checked]:icon-[fluent--megaphone-loud-20-filled]

  return (
    <Popover.Root autoFocus={false} open={!!checkboxProps.error} closeOnInteractOutside={false}>
      <Popover.Anchor class="flex flex-col">
        <Checkbox.Root {...rest} class={classes()}>
          <Checkbox.Control class="w-5 h-5">
            <Checkbox.Indicator class={iconClasses()} />
          </Checkbox.Control>
          <Checkbox.HiddenInput {...checkboxProps.inputProps} />
        </Checkbox.Root>
      </Popover.Anchor>
      <Portal>
        <Popover.Positioner>
          <Popover.Content class={`card ${props.error ? "card-error" : ""}`.trim()}>
            <p class="card-content px-4 p-2">{props.error}</p>
          </Popover.Content>
        </Popover.Positioner>
      </Portal>
    </Popover.Root>
  );
}
