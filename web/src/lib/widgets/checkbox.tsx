import { Checkbox, type CheckboxRootProps, Popover } from "@ark-ui/solid";
import { type JSX, Show, splitProps } from "solid-js";
import { Portal } from "solid-js/web";

export type CheckboxProps = {
  ghost?: boolean;
  error?: string;
  inputProps?: JSX.IntrinsicElements["input"];
};

export default function(
  props: CheckboxProps &
    CheckboxRootProps & {
      children?: JSX.Element;
    }
) {
  const [checkboxProps, _1] = splitProps(props, ["ghost", "error", "inputProps"]);
  const [{ children }, rest] = splitProps(_1, ["children"]);

  const classes = () =>
    `btn ${checkboxProps.ghost ? "btn-ghost" : ""} data-[state=checked]:border-2 data-[state=checked]:border-primary ${rest.disabled ? "btn-disabled" : ""}`.trim();
  return (
    <Popover.Root autoFocus={false} open={!!checkboxProps.error} closeOnInteractOutside={false}>
      <Popover.Anchor class={`flex flex-col space-y-1 flex-1 ${rest.class}`.trim()}>
        <Show when={props.title}>
          <label class="label" for={checkboxProps.inputProps?.id ?? "input_NOTPOSSIBLE"}>
            {props.title}
          </label>
        </Show>
        <Checkbox.Root {...rest} class={classes()}>
          <Checkbox.Label asChild={() => children} />
          <Checkbox.Control class="w-5 h-5">
            <Checkbox.Indicator class="data-[state=unchecked]:icon-[fluent--checkmark-circle-20-regular] data-[state=checked]:icon-[fluent--checkmark-circle-20-filled] data-[state=unchecked]:text-layer-content data-[state=checked]:text-primary !w-5 !h-5" />
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
