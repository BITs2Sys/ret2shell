import reservedBlank from "@lib/lifecycle/scripts/reserved_blank.rx";
import webhookExample from "@lib/lifecycle/scripts/webhook_example.rx";

export const lifecyclePresetMap = {
  "webhook-example": webhookExample,
  "reserved-blank": reservedBlank,
} as const;

export type LifecyclePreset = keyof typeof lifecyclePresetMap;

export const lifecyclePresetEntries = [
  {
    labelKey: "lifecycle.preset.webhookExample",
    value: "webhook-example",
  },
  {
    labelKey: "lifecycle.preset.reservedBlank",
    value: "reserved-blank",
  },
] as const satisfies ReadonlyArray<{
  labelKey: string;
  value: LifecyclePreset;
}>;
